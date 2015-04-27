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

public class BatchPacketTestAllocate {

  protected internal ConcurrentQueueBatch<Packet> queue_;

  private int producerCore_;
  private int consumerCore_;
  public long received_;
  public long produceBatchSize_;
  public long receiveBatchSize_;
  public BatchPacketTestAllocate(int producerCore, int consumerCore, int produceBatchSize, int receiveBatch) { 
    queue_ = new ConcurrentQueueBatch<Packet>();
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
      int dequed = queue_.DequeueBatch(ref batch);
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

public class BatchQueueTest {
  public static void Test () {
    ConcurrentQueueBatch<Int32> queue = new ConcurrentQueueBatch<Int32>();
    Int32[] batch = new Int32[10];
    for (int i = 0; i < batch.Length; i++) {
      batch[i] = i;
    }
    queue.EnqueueBatch(ref batch);
    Console.WriteLine("Is queue empty? " + queue.IsEmpty);
    while (!queue.IsEmpty) {
      Int32[] x = new Int32[20];
      int count = queue.DequeueBatch(ref x);
      Console.WriteLine("Dequed batch of " + count);
      for(int i = 0; i < count; i++) {
          Console.WriteLine("Dequeued " + x[i]);
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
    BatchPacketTestAllocate bp = new BatchPacketTestAllocate(0, 1, 500, 500);

    bp.Start();
  }
}
}
