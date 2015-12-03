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
	// Common interfaces for ports
	// Type for PCI information
	[StructLayout(LayoutKind.Sequential)]
	public struct PCIDevice {
		[StructLayout(LayoutKind.Sequential)]
		public struct TailQEntry {
			IntPtr next;
			IntPtr prev;
		}
		public TailQEntry Next;

		[StructLayout(LayoutKind.Sequential)]
		public struct PCIAddr {
			public ushort Domain;
			public byte   Bus;
			public byte   DevId;
			public byte   Function;
		}
		public PCIAddr Address;

		[StructLayout(LayoutKind.Sequential)]
		public struct PCIId {
			public ushort VendorId;
			public ushort DeviceId;
			public ushort SubsystemVendorId;
			public ushort SybsystemDeviceId;
		}
		public PCIId Id;

		[StructLayout(LayoutKind.Sequential)]
		public struct PCIResource {
			public ulong PhysicalAddress;
			public ulong Length;
			IntPtr Address;
		}
		// PCI_MAX_RESOURCE 6
		[MarshalAs(UnmanagedType.ByValArray, SizeConst = 6)]
		public PCIResource[] MemResource;

		[StructLayout(LayoutKind.Sequential)]
		public struct IntrHandleStruct {
			public int Fd;
			public int UioCfgFd;
			public int Type;
			public int MaxIntr;
			public uint NbEfd;
			public IntPtr IntrVec;
		}
		public IntrHandleStruct IntrHandle;

		public IntPtr Driver;
		public ushort MaxVfs;
		public int NumaNode;
		public IntPtr DevArgs;
		public int Kdrv; 

	}

	[StructLayout(LayoutKind.Sequential)]
	public struct EthDevInfo {
		private IntPtr pciDevPtr;
		public PCIDevice PCIDev {
			get {
				if (pciDevPtr != IntPtr.Zero)
					return (PCIDevice)Marshal.PtrToStructure(pciDevPtr, 
						typeof(PCIDevice)); 
				else
					return new PCIDevice();
			}
		}

		private IntPtr driverNamePtr;
		public string DriverName {
			get { return Marshal.PtrToStringAnsi(driverNamePtr); }
		}

		public uint IfIndex;
		
		public uint MinRxBufsize;
		public uint MaxRxPktlen;
		public ushort MaxRxQueues;
		public ushort MaxTxQueues;

		public uint MaxMacAddrs;
		public uint MaxHashMacAddrs;

		public ushort MaxVFs;
		public ushort MaxVmdqPools;

		public uint RxOffloadCapabilities;
		public uint TxOffloadCapabilities;

		public ushort RetaSize;

		public byte HashKeySize;

		public ulong FlowTypeRSSOffloads;
		
		[StructLayout(LayoutKind.Sequential)]
		public struct EthThresh {
			public byte PThresh;
			public byte HThresh;
			public byte WThtresh;
		}

		[StructLayout(LayoutKind.Sequential)]
		public struct EthRxconf {
			public EthThresh RxThresh;
			ushort RxFreeThresh;
			byte RxDropEn;
			byte RxDeferredStart;
		}
		public EthRxconf DefaultRxConf;

		[StructLayout(LayoutKind.Sequential)]
		public struct EthTxconf {
			public EthThresh TxThresh;
			public ushort TxRsThresh;
			public ushort TxFreeThresh;
			public uint TxqFlags;
			public byte TxDeferredStart;
		}
		public EthTxconf DefaultTxConf;

		public ushort VmdqQueueBase;
		public ushort VmdqQueueNum;
		public ushort VmdqPoolBase;
	}

	public class PMDPorts {
		public static int SizeofEthDevInfo() {
			return Marshal.SizeOf(typeof(EthDevInfo));
		}

		[ReliabilityContract(Consistency.WillNotCorruptState, Cer.Success)]
		[DllImport("zcsi")]
		private static extern int num_pmd_ports();

		[ReliabilityContract(Consistency.WillNotCorruptState, Cer.Success)]
		[DllImport("zcsi")]
		private static extern int get_pmd_ports([In, Out] EthDevInfo[] info,
				int len);

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public static int NumPMDPorts() {
			return num_pmd_ports();
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public static EthDevInfo[] GetPMDPortInfo() {
			int ports = NumPMDPorts();
			EthDevInfo[] portInfo = new EthDevInfo[ports];
			for (int i = 0; i < ports; i++) {
				portInfo[i] = new EthDevInfo();
			}
			int ret = get_pmd_ports(portInfo, ports);
			Debug.Assert(ret == ports);
			return portInfo;
		}
	}
}
