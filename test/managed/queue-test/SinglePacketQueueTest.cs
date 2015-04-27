using System;
using System.Collections.Concurrent;
using System.Threading;
using System.Diagnostics;

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
  public long received_;
  public SinglePacketTestAllocate(int producerCore, int consumerCore) { 
    queue_ = new ConcurrentQueue<Packet>();
    producerCore_ = producerCore;
    consumerCore_ = consumerCore;
    received_ = 0;
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
    long lastElapsed = stopwatch.ElapsedMilliseconds;
    long count = 0;
    while (true) {
      Packet pkt;
      if (queue_.TryDequeue(out pkt)) {
        count++;
        received_ = pkt.id;
      }
      long currSec = SysUtils.GetSecond(stopwatch);
      if (currSec != lastSec) {
        lastSec = currSec;
        long currElapsed = stopwatch.ElapsedMilliseconds;
        Console.WriteLine(SysUtils.GetCurrentCpu() + " " 
                        + count + " " + received_ + " " + (currElapsed - lastElapsed));
        lastElapsed = currElapsed;
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


#if false
public class QueueTest {
  public static void Main (string[] args) {
    #if __MonoCS__
    Console.WriteLine("Running Mono");
    #else
    Console.WriteLine("Running Windows");
    #endif

    #if REUSE
    Console.WriteLine("Reusing buffers");
    SinglePacketTestReuse sp = new SinglePacketTestReuse(0, 1, 10000000);
    #else
    Console.WriteLine("Allocating new packets");
    SinglePacketTestAllocate sp = new SinglePacketTestAllocate(0, 1);
    #endif
    
    sp.Start();
  }
}
#endif
}
