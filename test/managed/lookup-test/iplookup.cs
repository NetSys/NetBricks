using System;
using System.Collections;
using System.Collections.Generic;
using System.Diagnostics;
using System.Diagnostics.Contracts;
using System.IO;
using System.Net;

namespace E2D2.Collections {
  class IPLookup {
    private UInt16[] tbl24_;
    private UInt16[] tblLong_;
    private Dictionary<UInt32, UInt16>[] prefixTable_;
    private uint currentTBLLong_;
    const uint TBL24_SIZE = (1u << 24) + 1;
    const uint TBLLONG_SIZE = (1u << 24) + 1;
    const UInt16 OVERFLOW_MASK = 0x8000;

    public IPLookup() {
      tbl24_ = new UInt16[TBL24_SIZE];
      tblLong_ = new UInt16[TBLLONG_SIZE];
      currentTBLLong_ = 0;
      prefixTable_ = new Dictionary<UInt32, UInt16>[33];
      for(int i = 0; i < prefixTable_.Length; i++) {
        prefixTable_[i] = new Dictionary<UInt32, UInt16>();
      }
    }

    public void AddRoute(UInt32 address, UInt16 length, UInt16 nextHop) {
      prefixTable_[length][address] = nextHop;
    }

    public void DeleteRoute(UInt32 address, UInt16 length) {
      prefixTable_[length].Remove(address);
    }

    public void ConstructFIB() {
      // Order gives us LPM.
      for (int i = 0; i <= 24; i++) {
        foreach(var it in prefixTable_[i]) {
          UInt32 address = it.Key;
          UInt16 dest = it.Value;
          UInt32 start = address >> 8;
          // Each row in table represents a 24 bit range, so compute
          // the number of 24 bit prefixes to order.
          UInt32 end = start + (1u << (24 - i));
          for (UInt32 pfx = start; pfx < end; pfx++) {
            tbl24_[pfx] = dest;
          }
        }
      }
      // For these ones we want to fill up the overflow table too.
      for (int i = 25; i <= 32; i++) {
        foreach(var it in prefixTable_[i]) {
          UInt32 addr = it.Key;
          UInt16 dest = it.Value;
          UInt16 tblDest = tbl24_[addr >> 8]; // Check what the main table currently has.
          // See if we have already spilled over for these 24 bits.
          if ((tblDest & OVERFLOW_MASK) == 0) {
            // We assign 256 entries for each overflow 24-bit prefix
            UInt32 start = currentTBLLong_ + (addr & 0xff); // The last 8-bits of the address.
            UInt32 end = start + (1u << (32 - i)); // How many of the 8-bit entries need to be filled.
            for (UInt32 j = currentTBLLong_; j < currentTBLLong_ + 256; j++) {
              if (j < start || j >= end) {
                tblLong_[j] = tblDest; // When outside the range copy the old value
              } else {
                tblLong_[j] = dest;
              }
              tbl24_[addr >> 8] = 
                  (UInt16)((currentTBLLong_ >> 8) | OVERFLOW_MASK);
            }
          } else {
              UInt32 start = (((UInt32)(tblDest & (~OVERFLOW_MASK)) << 8) + (addr & 0xff));
              UInt32 end = start + (1u << (32 - i));
              for (UInt32 j = start; j < end; j++) {
                tblLong_[j] = dest;
              }
          }
        }
      }
    }
    
    public UInt16 RouteLookup(UInt32 ip) {
      UInt16 tblDest = tbl24_[ip >> 8];
      if ((tblDest & OVERFLOW_MASK) > 0) {
        int index = (int)(((UInt32)(tblDest & (~OVERFLOW_MASK)) << 8) + (ip & 0xff));
        return tblLong_[index];
      } else {
        return tblDest;
      }
    }
  }
  public class ThroughputTest {
    static void Benchmark(ref IPLookup lookup, 
            ref UInt32[] trace, 
            long warm,
            long batch,
            long batches) {
      lookup.ConstructFIB();
      Stopwatch stopwatch = new Stopwatch();
      stopwatch.Start();
      long lastSec = SysUtils.GetSecond(stopwatch);
      long lastElapsed = stopwatch.ElapsedMilliseconds;
      UInt32 lookups = 0;
      int length = trace.Length;
      while (SysUtils.GetSecond(stopwatch) - lastSec < warm) {
        for (int i = 0; i < batch; i++) {
          lookup.RouteLookup(trace[lookups % length]);
          lookups++;
        }
      }
      lastSec = SysUtils.GetSecond(stopwatch);
      lastElapsed = stopwatch.ElapsedMilliseconds;
      long lastLookups = 0;
      long tested = 0;
      while (tested < batches) {
        for (int i = 0; i < batch; i++) {
          lookup.RouteLookup(trace[lookups % length]);
          lookups++;
          lastLookups++;
        }
        long currSec = SysUtils.GetSecond(stopwatch);
        if (currSec != lastSec) {
          tested++;
          long currElapsed = stopwatch.ElapsedMilliseconds;
          long elapsedMs = (currElapsed - lastElapsed);
          long elapsedSec = (currElapsed - lastElapsed) / 1000;
          Console.WriteLine(elapsedSec + " " + elapsedMs + " " + batch + " " +
              lastLookups/elapsedSec);
          lastElapsed = currElapsed;
          lastLookups = 0;
          lastSec = currSec;
        }
      }
    }
    static void Main(string[] args) {
      if(args.Length < 2) {
        Console.WriteLine("Usage: IPLookup <rib> <trace>");
        return;
      }
      List<UInt32> trace = new List<UInt32>();
      IPLookup lookup = new IPLookup();
      StreamReader ribReader = new StreamReader(args[0]);
      while (ribReader.Peek() >= 0) {
        String line = ribReader.ReadLine();
        String[] parts = line.Split(' ');
        String[] addrParts = parts[0].Split('/');
        UInt16 dest = Convert.ToUInt16(parts[1]);
        UInt16 len = Convert.ToUInt16(addrParts[1]);
        IPAddress addr = IPAddress.Parse(addrParts[0]);
        UInt32 addrAsInt = 
            (UInt32)IPAddress.NetworkToHostOrder(
                    BitConverter.ToInt32(
                        addr.GetAddressBytes(), 0));
        lookup.AddRoute(addrAsInt, len, dest);
      }
      lookup.ConstructFIB();
      ribReader.Close();
      StreamReader traceReader = new StreamReader(args[1]);
      while (traceReader.Peek() >= 0) {
          UInt32 address = Convert.ToUInt32(traceReader.ReadLine());
          trace.Add(address);
      }
      UInt32[] traceArray = trace.ToArray();
      trace = null;
      const long WARM = 5;
      const int BATCH_SIZE = 10;
      const long BATCHES = 50;
      for (int bexp = 0; bexp < BATCH_SIZE; bexp++) {
        Benchmark(ref lookup, ref traceArray, WARM, (1L << bexp), BATCHES);
      }
    }
  }
}
