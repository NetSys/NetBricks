using System;
using System.Collections;
using System.Collections.Generic;
using System.Diagnostics;
using System.Diagnostics.Contracts;

namespace E2D2.Collections {
  class IPLookup {
    private UInt16[] tbl24_;
    private UInt16[] tblLong_;
    private Dictionary<UInt32, UInt16>[] prefixTable_;
    private uint currentTBLLong_;
    const uint TBL24_SIZE = (1u << 24) + 1;
    const uint TBLLONG_SIZE = (1u << 24) + 1;
    const UInt16 OVERFLOW_MASK = 0x8000u;

    public IPLookup() {
      tbl24_ = new UInt16[TBL24_SIZE];
      tblLong_ = new UInt16[TBLLONG_SIZE];
      currentTBLLong_ = 0;
      prefixTable_ = new Dictionary<UInt32, UInt16>[33];
      for(int i = 0; i < prefixTable_.Length; i++) {
        prefixTable_[i] = new Dictionary<UInt32, UInt16>();
      }
    }

    public void AddRoute(UInt32 address, UInt16 length; UInt16 nextHop) {
      prefixTable_[length][address] = nextHop;
    }

    public void DeleteRoute(UInt32 address, UInt16 length) {
      prefixTable_[length].Remove(address);
    }

    public void ConstructFIB() {
      // Order gives us LPM.
      for (uint i = 0; i <= 24; i++) {
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
      for (uint i = 25; i <= 32; i++) {
        foreach(var it in prefixTable_[i]) {
          UInt32 addr = it.Key;
          UInt16 dest = it.Value;
          UInt32 tblDest = tbl24_[addr >> 8]; // Check what the main table currently has.
          // See if we have already spilled over for these 24 bits.
          if (tblDest & OVERFLOW_MASK) {
            // We assign 256 entries for each overflow 24-bit prefix
            UInt32 start = currentTBLLong_ + (addr & 0xff); // The last 8-bits of the address.
            UInt32 end = start + (1u << (32 - i)); // How many of the 8-bit entries need to be filled.
            for (UInt32 j = currentTBLLong_; j < currentTBLLong_ + 256; j++) {
              if (j < start || j >= end) {
                tblLong_[j] = tblDest; // When outside the range copy the old value
              } else {
                tblLong_[j] = dest;
              }
              tbl24_[addr >> 8] = (currentTBLLong_ >> 8) | OVERFLOW_MASK;
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
      if (tblDest & OVERFLOW_MASK) {
        int index = (int)(((UInt32)(tblDest & (~OVERFLOW_MASK)) << 8) + (ip & 0xff));
        return tblLong_[index];
      } else {
        return tblDest;
      }
    }
  }
  public class ThroughputTest {
    static void Main(string[] args) {
      IPLookup lookup = new IPLookup();
    }
  }
}
