using System;
using System.Runtime.InteropServices;
using System.Windows.Forms;

namespace ReLogic.Localization.IME.Windows;

internal static class NativeMethods
{
	public struct INPUT
	{
		public uint Type;

		public INPUTUNION Data;
	}

	[StructLayout(LayoutKind.Explicit)]
	public struct INPUTUNION
	{
		[FieldOffset(0)]
		public MOUSEINPUT MouseInput;

		[FieldOffset(0)]
		public KEYBDINPUT KeyboardInput;

		[FieldOffset(0)]
		public HARDWAREINPUT HardwareInput;
	}

	public struct KEYBDINPUT
	{
		public ushort VirtualKey;

		public ushort Scan;

		public uint Flags;

		public uint Time;

		public IntPtr ExtraInfo;
	}

	public struct HARDWAREINPUT
	{
		public uint Msg;

		public ushort ParamL;

		public ushort ParamH;
	}

	public struct MOUSEINPUT
	{
		public int X;

		public int Y;

		public uint MouseData;

		public uint Flags;

		public uint Time;

		public IntPtr ExtraInfo;
	}

	private const string DLL_NAME = "ReLogic.Native.dll";

	[DllImport("ReLogic.Native.dll", CharSet = CharSet.Unicode)]
	[return: MarshalAs(UnmanagedType.I1)]
	public static extern bool ImeUi_Initialize(IntPtr hWnd, [MarshalAs(UnmanagedType.I1)] bool bDisabled = false);

	[DllImport("ReLogic.Native.dll", CharSet = CharSet.Unicode)]
	[return: MarshalAs(UnmanagedType.I1)]
	public static extern bool ImeUi_Uninitialize();

	[DllImport("ReLogic.Native.dll", CharSet = CharSet.Unicode, EntryPoint = "ImeUi_EnableIme")]
	public static extern void ImeUi_Enable([MarshalAs(UnmanagedType.I1)] bool bEnable);

	[DllImport("ReLogic.Native.dll", CharSet = CharSet.Unicode)]
	[return: MarshalAs(UnmanagedType.I1)]
	public static extern bool ImeUi_IsEnabled();

	[DllImport("ReLogic.Native.dll", CharSet = CharSet.Unicode)]
	public static extern IntPtr ImeUi_ProcessMessage(IntPtr hWnd, int msg, IntPtr wParam, ref IntPtr lParam, [MarshalAs(UnmanagedType.I1)] ref bool trapped);

	[DllImport("ReLogic.Native.dll", CharSet = CharSet.Unicode)]
	public static extern IntPtr ImeUi_GetCompositionString();

	[DllImport("ReLogic.Native.dll", CharSet = CharSet.Unicode)]
	public static extern IntPtr ImeUi_GetCandidate(uint index);

	[DllImport("ReLogic.Native.dll", CharSet = CharSet.Unicode)]
	public static extern uint ImeUi_GetCandidateSelection();

	[DllImport("ReLogic.Native.dll", CharSet = CharSet.Unicode)]
	public static extern void ImeUi_SetCandidateSelection(uint index);

	[DllImport("ReLogic.Native.dll", CharSet = CharSet.Unicode)]
	public static extern uint ImeUi_GetCandidateCount();

	[DllImport("ReLogic.Native.dll", CharSet = CharSet.Unicode)]
	public static extern void ImeUi_FinalizeString([MarshalAs(UnmanagedType.I1)] bool bSend = false);

	[DllImport("ReLogic.Native.dll", CharSet = CharSet.Unicode)]
	public static extern uint ImeUi_GetCandidatePageSize();

	[DllImport("ReLogic.Native.dll", CharSet = CharSet.Unicode, EntryPoint = "ImeUi_IsShowCandListWindow")]
	[return: MarshalAs(UnmanagedType.I1)]
	public static extern bool ImeUi_IsCandidateListVisible();

	[DllImport("ReLogic.Native.dll", CharSet = CharSet.Unicode, EntryPoint = "ImeUi_IgnoreHotKey")]
	[return: MarshalAs(UnmanagedType.I1)]
	public static extern bool ImeUi_ShouldIgnoreHotKey(ref Message message);

	[DllImport("ReLogic.Native.dll")]
	public static extern ushort ImeUi_GetPrimaryLanguage();

	[DllImport("Imm32.dll")]
	public static extern uint ImmGetVirtualKey(IntPtr hWnd);

	[DllImport("user32.dll")]
	public static extern int SendInput(int nInputs, INPUT[] pInputs, int cbSize);
}
