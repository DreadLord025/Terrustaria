using System;
using System.Runtime.InteropServices;

namespace ReLogic.Peripherals.RGB.Corsair;

internal class NativeMethods
{
	private const string DLL_NAME = "CUESDK_2015.dll";

	[DllImport("CUESDK_2015.dll", CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Unicode)]
	[return: MarshalAs(UnmanagedType.I1)]
	public static extern bool CorsairSetLedsColors(int size, [In][Out] CorsairLedColor[] ledsColors);

	[DllImport("CUESDK_2015.dll", CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Unicode)]
	[return: MarshalAs(UnmanagedType.I1)]
	public static extern bool CorsairSetLedsColorsAsync(int size, [In][Out] CorsairLedColor[] ledsColors, IntPtr callback, IntPtr context);

	[DllImport("CUESDK_2015.dll", CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Unicode)]
	public static extern int CorsairGetDeviceCount();

	[DllImport("CUESDK_2015.dll", CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Unicode)]
	public static extern IntPtr CorsairGetDeviceInfo(int deviceIndex);

	[DllImport("CUESDK_2015.dll", CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Unicode)]
	public static extern IntPtr CorsairGetLedPositions();

	[DllImport("CUESDK_2015.dll", CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Unicode)]
	public static extern IntPtr CorsairGetLedPositionsByDeviceIndex(int deviceIndex);

	[DllImport("CUESDK_2015.dll", CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Unicode)]
	public static extern bool CorsairRequestControl(CorsairAccessMode accessMode);

	[DllImport("CUESDK_2015.dll", CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Unicode)]
	public static extern CorsairProtocolDetails CorsairPerformProtocolHandshake();

	[DllImport("CUESDK_2015.dll", CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Unicode)]
	public static extern CorsairError CorsairGetLastError();

	[DllImport("CUESDK_2015.dll", CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Unicode)]
	public static extern bool CorsairReleaseControl(CorsairAccessMode accessMode);
}
