using System;
using System.Collections.Concurrent;
using System.Threading;
using System.Diagnostics;
using E2D2.Collections.Concurrent;

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

public class RingThroughputTestNoAllocate {

  protected internal LLRing<Packet> queue_;

  private int producerCore_;
  private int consumerCore_;
  public long received_;
  public long produceBatchSize_;
  public long receiveBatchSize_;
  public RingThroughputTestNoAllocate(int producerCore, 
                                 int consumerCore, 
                                 uint ringSize,
                                 int produceBatchSize, 
                                 int receiveBatch) { 
    queue_ = new LLRing<Packet>(ringSize, false, false);
    producerCore_ = producerCore;
    consumerCore_ = consumerCore;
    received_ = 0;
    produceBatchSize_ = produceBatchSize;
    receiveBatchSize_ = receiveBatch;
  }

  protected void ProducerStart() {
    SysUtils.SetAffinity(producerCore_);
    Stopwatch stopwatch = new Stopwatch();
    stopwatch.Start();
    long lastSec = SysUtils.GetSecond(stopwatch);
    long count = 0;
    long absCount = 0;
    Packet[] batch = new Packet[produceBatchSize_]; 
    for (int i = 0; i < batch.Length; i++) {
        batch[i] = new Packet(absCount);
        absCount++;
    }
    while (true) {
      long currSec = SysUtils.GetSecond(stopwatch);
      count++;
      queue_.EnqueueBatch(ref batch);
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
    Packet[] batch = new Packet[receiveBatchSize_];
    while (true) {
      int dequed = (int)queue_.DequeueBatch(ref batch);
      
      if (dequed > 0) {
        received_ = batch[dequed - 1].id;
      }
      count += dequed;
      long currSec = SysUtils.GetSecond(stopwatch);
      if (currSec != lastSec) {
        lastSec = currSec;
        long currElapsed = stopwatch.ElapsedMilliseconds;
        Console.WriteLine(SysUtils.GetCurrentCpu() + " " + dequed + " "  
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
public class RingThroughputTest {
  public static void Test () {
      LLRing<Int64> llring = new LLRing<Int64>(256, true, true);
      Int64[] arr = new Int64[257];
      Int64[] arr2 = new Int64[25];
      for (int i = 0; i < arr.Length; i++) {
          arr[i] = i;
      }
      UInt32 ret = llring.EnqueueBatch(ref arr);
      Debug.Assert((ret & (~LLRing<Int64>.RING_QUOT_EXCEED)) < 256);
      Console.WriteLine("Enqueued " + ret);
      while ((ret = llring.DequeueBatch(ref arr2)) != 0) {
          for (int i = 0; i < ret; i++) {
              Console.WriteLine("Dequeued " + arr2[i]);
          }
      }
    
  }
  public static void Main (string[] args) {
    #if __MonoCS__
    Console.WriteLine("Running Mono");
    #else
    Console.WriteLine("Running Windows");
    #endif
    //Test();
    //
    RingThroughputTestNoAllocate rt = new RingThroughputTestNoAllocate(0, 1, (1u << 12), 1024, 512);
    rt.Start();
  }
}
}
