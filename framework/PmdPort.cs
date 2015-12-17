using System;
using System.Collections.Generic;
using System.Collections;
using System.Diagnostics;
using System.Diagnostics.Contracts;
using System.Runtime.CompilerServices;
using System.Runtime.ConstrainedExecution;
using System.Runtime.InteropServices;
using System.Text;
using System.Security;

namespace ZCSI.DPDK {
	public sealed class PMDPort : IDisposable {
		[DllImport("zcsi")]
		private static extern int init_pmd_port(int port,
				int rxqs, int txqs, [In]int[] rxcores,
				[In]int[] txcores, int nrxd, int ntxd,
				int loopback, int tso, int csumoffload);

		[ReliabilityContract(Consistency.WillNotCorruptState, Cer.Success)]
		[DllImport("zcsi")]
		private static extern int free_pmd_port(int port);

		[ReliabilityContract(Consistency.WillNotCorruptState, Cer.Success)]
		[DllImport("zcsi")]
		private static extern int recv_pkts(int port, int qid, IntPtr array, int len);

		[ReliabilityContract(Consistency.WillNotCorruptState, Cer.Success)]
		[DllImport("zcsi")]
		private static extern int send_pkts(int port, int qid, IntPtr array, int len);

		public const int DEFAULT_RXQUEUE_SIZE = 128;
		public const int DEFAULT_TXQUEUE_SIZE = 512;

		private bool _open;
		private int _port;

		private int _rxqs;
		private int _txqs;

		public PMDPort(int port, int rxqs, int txqs, int[] rxcores,
				int[] txcores, int nrxd, int ntxd, bool loopback,
				bool tso, bool csumoffload) {
			if (rxcores.Length != rxqs) {
				throw new ArgumentException(
					"Must specify as many cores as rxqs");
			}

			if (txcores.Length != rxqs) {
				throw new ArgumentException(
					"Must specify as many cores as txqs");
			}

			init_pmd_port(port, rxqs, txqs, rxcores, txcores, nrxd,
				ntxd, (loopback? 1 : 0), (tso ? 1 : 0),
				(csumoffload ? 1 : 0));
			_open = true;
			_port = port;
			_rxqs = rxqs;
			_txqs = txqs;
		}

		public PMDPort(int port, int rxcore, int txcore, int nrxd,
			int ntxd, bool loopback, bool tso, bool csumoffload) :
		this(port, 1, 1, new int[]{rxcore}, new int[]{txcore}, nrxd,
		     ntxd, loopback, tso, csumoffload)
		{
		}
		
		public static PMDPort LoopbackPort(int port, int core) {
			return new PMDPort(port, core, core, DEFAULT_RXQUEUE_SIZE,
					DEFAULT_TXQUEUE_SIZE, true, false, false);
		}

		public static PMDPort Port(int port, int core) {
			return new PMDPort(port, core, core, DEFAULT_RXQUEUE_SIZE,
					DEFAULT_TXQUEUE_SIZE, false, false, false);
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public void Close() {
			if (_open) {
				_open = false;
				free_pmd_port(_port);
			}
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public void ReceivePacketBatch(PacketBatch batch, int len, int queue) {
			Contract.Requires(_open);
			Contract.Requires(batch != null);
			Contract.Requires(queue < _rxqs);

			len = Math.Min(batch._length, len);
			len = recv_pkts(_port, queue, batch._packetPointers, len);
			batch._available = len;
			batch._start = 0;
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public PacketBatch ReceivePacketBatch(int len, int queue) {
			PacketBatch batch = new PacketBatch(len);
			ReceivePacketBatch(batch, len, queue);
			return batch;
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public int SendPacketBatch(PacketBatch batch, int queue) {
			Contract.Requires(_open);
			Contract.Requires(batch != null);
			Contract.Requires(queue < _txqs);
			int len = send_pkts(_port, queue, batch._packetPointers, batch._available);
			batch._start = len;
			return len;
		}

		[ReliabilityContract(Consistency.WillNotCorruptState, Cer.Success)]
		public void Dispose() {
			Dispose(true);
			GC.SuppressFinalize(this);
		}

		[ReliabilityContract(Consistency.WillNotCorruptState, Cer.Success)]
		void Dispose(bool disposing) {
			Close();
		}

		[ReliabilityContract(Consistency.WillNotCorruptState, Cer.Success)]
		~PMDPort() {
			Dispose(false);
		}
	}
}
