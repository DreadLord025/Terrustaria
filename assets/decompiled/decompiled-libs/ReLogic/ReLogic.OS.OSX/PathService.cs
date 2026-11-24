using System;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Text;
using ReLogic.OS.Base;

namespace ReLogic.OS.OSX;

internal class PathService : ReLogic.OS.Base.PathService
{
	private class NativeMethods
	{
		private const string Foundation = "/System/Library/Frameworks/Foundation.framework/Foundation";

		[DllImport("/System/Library/Frameworks/Foundation.framework/Foundation", BestFitMapping = false, CharSet = CharSet.Ansi, ThrowOnUnmappableChar = true)]
		public static extern IntPtr objc_getClass([MarshalAs(UnmanagedType.LPStr)] string name);

		[DllImport("/System/Library/Frameworks/Foundation.framework/Foundation", BestFitMapping = false, CharSet = CharSet.Ansi, ThrowOnUnmappableChar = true)]
		public static extern IntPtr sel_registerName([MarshalAs(UnmanagedType.LPStr)] string name);

		[DllImport("/System/Library/Frameworks/Foundation.framework/Foundation")]
		public static extern IntPtr objc_msgSend(IntPtr receiver, IntPtr selector);

		[DllImport("/System/Library/Frameworks/Foundation.framework/Foundation")]
		public static extern IntPtr objc_msgSend(IntPtr receiver, IntPtr selector, IntPtr arg1);

		[DllImport("/System/Library/Frameworks/Foundation.framework/Foundation")]
		public static extern IntPtr objc_msgSend(IntPtr receiver, IntPtr selector, IntPtr arg1, IntPtr arg2, IntPtr arg3);
	}

	public override string GetStoragePath()
	{
		string environmentVariable = Environment.GetEnvironmentVariable("HOME");
		if (string.IsNullOrEmpty(environmentVariable))
		{
			return ".";
		}
		return environmentVariable + "/Library/Application Support";
	}

	public override void OpenURL(string url)
	{
		Process.Start("open", "\"" + url + "\"");
	}

	private static IntPtr MarshalNSString(string str)
	{
		byte[] bytes = Encoding.UTF8.GetBytes(str + "\0");
		IntPtr intPtr = Marshal.AllocHGlobal(bytes.Length);
		try
		{
			Marshal.Copy(bytes, 0, intPtr, bytes.Length);
			IntPtr receiver = NativeMethods.objc_getClass("NSString");
			IntPtr selector = NativeMethods.sel_registerName("stringWithUTF8String:");
			return NativeMethods.objc_msgSend(receiver, selector, intPtr);
		}
		finally
		{
			Marshal.FreeHGlobal(intPtr);
		}
	}

	public override void MoveToRecycleBin(string path)
	{
		IntPtr receiver = NativeMethods.objc_getClass("NSFileManager");
		IntPtr selector = NativeMethods.sel_registerName("defaultManager");
		IntPtr receiver2 = NativeMethods.objc_msgSend(receiver, selector);
		IntPtr receiver3 = NativeMethods.objc_getClass("NSURL");
		IntPtr selector2 = NativeMethods.sel_registerName("fileURLWithPath:");
		IntPtr arg = MarshalNSString(path);
		IntPtr arg2 = NativeMethods.objc_msgSend(receiver3, selector2, arg);
		IntPtr selector3 = NativeMethods.sel_registerName("trashItemAtURL:resultingItemURL:error:");
		NativeMethods.objc_msgSend(receiver2, selector3, arg2, IntPtr.Zero, IntPtr.Zero);
	}
}
