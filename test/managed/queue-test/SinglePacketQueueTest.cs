using System;
using System.Collections.Concurrent;
using System.Threading;
using System.Runtime.InteropServices;
using System.Diagnostics;
#if (!__MonoCS__)
using System.Linq;
#endif

namespace E2D2 {
public class Packet {
  Int64 id_;
  public Int64 id 
  {
      get {return id_;}
  }
  public Packet(Int64 id) {
    id_ = id;
  }
}

public class SinglePacketTestAllocate {

  protected internal ConcurrentQueue<Packet> queue_;

  private int producerCore_;
  private int consumerCore_;
  public SinglePacketTestAllocate(int producerCore, int consumerCore) { 
    queue_ = new ConcurrentQueue<Packet>();
    producerCore_ = producerCore;
    consumerCore_ = consumerCore;
  }

  protected void ProducerStart() {
    SysUtils.SetAffinity(producerCore_);
    Stopwatch stopwatch = new Stopwatch();
    stopwatch.Start();
    long lastSec = SysUtils.GetSecond(stopwatch);
    long count = 0;
    long absCount = 0;
    while (true) {
      long currSec = SysUtils.GetSecond(stopwatch);
      queue_.Enqueue(new Packet(absCount));
      count++;
      absCount++;
      if (currSec != lastSec) {
        lastSec = currSec;
        count = 0;
      }
    }
  }

  protected void ConsumerStart() {
    SysUtils.SetAffinity(consumerCore_);
    Stopwatch stopwatch = new Stopwatch();
    stopwatch.Start();
    long lastSec = SysUtils.GetSecond(stopwatch);
    long count = 0;
    long received = 0;
    while (true) {
      Packet pkt;
      if (queue_.TryDequeue(out pkt)) {
        count++;
        received = pkt.id;
      }
      long currSec = SysUtils.GetSecond(stopwatch);
      if (currSec != lastSec) {
        lastSec = currSec;
        Console.WriteLine("Consume " + SysUtils.GetCurrentCpu() + " " + count + " " + received);
        count = 0;
      }
    }
  }

  public void Start() {
    Thread producer = new Thread(new ThreadStart(this.ProducerStart));
    Thread consumer = new Thread(new ThreadStart(this.ConsumerStart));
    producer.Start();
    consumer.Start();
    producer.Join();
    consumer.Join();
  }

}

public class QueueTest {
  public static void Main (string[] args) {
    #if __MonoCS__
    Console.WriteLine("Decided this is Mono");
    #else
    Console.WriteLine("Windows");
    #endif
    SinglePacketTestAllocate sp = new SinglePacketTestAllocate(0, 1);
    sp.Start();
  }
}
}
