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
  public long seconds_;
  public RingThroughputTestNoAllocate(int producerCore, 
                                 int consumerCore, 
                                 uint ringSize,
                                 int produceBatchSize, 
                                 int receiveBatch,
                                 long count) { 
    queue_ = new LLRing<Packet>(ringSize, false, false);
    producerCore_ = producerCore;
    consumerCore_ = consumerCore;
    received_ = 0;
    produceBatchSize_ = produceBatchSize;
    receiveBatchSize_ = receiveBatch;
    seconds_ = count;
  }

  protected void ProducerStart() {
    SysUtils.SetAffinity(producerCore_);
    Stopwatch stopwatch = new Stopwatch();
    stopwatch.Start();
    long absCount = 0;
    Packet[] batch = new Packet[produceBatchSize_]; 
    for (int i = 0; i < batch.Length; i++) {
        batch[i] = new Packet(absCount);
        absCount++;
    }
    while (true) {
      queue_.EnqueueBatch(ref batch);
    }
  }

  protected void ConsumerStart() {
    SysUtils.SetAffinity(consumerCore_);
    Stopwatch stopwatch = new Stopwatch();
    stopwatch.Start();
    long lastSec = SysUtils.GetSecond(stopwatch);
    long lastElapsed = stopwatch.ElapsedMilliseconds;
    long count = 0;
    long seconds = 0;
    Packet[] batch = new Packet[receiveBatchSize_];
    long batchSize = receiveBatchSize_;
    while (true) {
      int dequed = (int)queue_.DequeueBatch(ref batch);
      
      if (dequed > 0) {
        received_ = batch[dequed - 1].id;
      }
      count += dequed;
      long currSec = SysUtils.GetSecond(stopwatch);
      if (currSec != lastSec) {
        seconds += (currSec - lastSec);
        lastSec = currSec;
        if (seconds >= seconds_) {
            long currElapsed = stopwatch.ElapsedMilliseconds;
            Console.WriteLine(SysUtils.GetCurrentCpu() + " " 
                + batchSize + " " + dequed + " "  
                + count + " " + received_ + " " 
                + (currElapsed - lastElapsed));
            seconds = 0;
            count = 0;
            lastElapsed = currElapsed;
            return;
        }
      }
    }
  }

  public void Start() {
    Thread producer = new Thread(new ThreadStart(this.ProducerStart));
    Thread consumer = new Thread(new ThreadStart(this.ConsumerStart));
    producer.Start();
    consumer.Start();
    consumer.Join();
    producer.Abort();
    producer.Join();
  }
}
public class RingThroughputTest {
  public static void Main (string[] args) {
    #if __MonoCS__
    Console.WriteLine("Running Mono");
    #else
    Console.WriteLine("Running Windows");
    #endif
    //Test();
    RingThroughputTestNoAllocate jit = new RingThroughputTestNoAllocate(1, 2, 512u, 2, 2, (1 << 4));
    jit.Start();
    // Actual test
    for (int i = 0; i <= 11; i++) {
        int buffer = (1<<i);
        RingThroughputTestNoAllocate rt = new RingThroughputTestNoAllocate(1, 2, 512u, buffer, buffer, (1 << 6));
        rt.Start();
    }
  }
}
}
