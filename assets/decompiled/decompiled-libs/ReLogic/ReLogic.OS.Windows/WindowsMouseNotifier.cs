using System;
using System.Runtime.InteropServices;
using System.Windows.Forms;

namespace ReLogic.OS.Windows;

internal class WindowsMouseNotifier : IMessageFilter, IMouseNotifier
{
	private const int WM_DEVICECHANGE = 537;

	private const int DBT_DEVNODES_CHANGED = 7;

	private bool mouseAttached = true;

	internal event Action<bool> MouseStateChanged;

	internal WindowsMouseNotifier(WindowsMessageHook wndProc)
	{
		wndProc.AddMessageFilter(this);
	}

	public void AddMouseHandler(Action<bool> action)
	{
		MouseStateChanged += action;
	}

	public void RemoveMouseHandler(Action<bool> action)
	{
		MouseStateChanged -= action;
	}

	public void ForceCursorHidden()
	{
		for (int num = NativeMethods.ShowCursor(bShow: false); num > 0; num = NativeMethods.ShowCursor(bShow: false))
		{
		}
	}

	private bool HasMouseAttached()
	{
		uint puiNumDevices = 0u;
		NativeMethods.GetRawInputDeviceList(null, ref puiNumDevices, (uint)Marshal.SizeOf(typeof(NativeMethods.RAWINPUTDEVICELIST)));
		NativeMethods.RAWINPUTDEVICELIST[] array = new NativeMethods.RAWINPUTDEVICELIST[puiNumDevices];
		NativeMethods.GetRawInputDeviceList(array, ref puiNumDevices, (uint)Marshal.SizeOf(typeof(NativeMethods.RAWINPUTDEVICELIST)));
		bool result = false;
		NativeMethods.RAWINPUTDEVICELIST[] array2 = array;
		for (int i = 0; i < array2.Length; i++)
		{
			if (array2[i].dwType == 0)
			{
				result = true;
				break;
			}
		}
		return result;
	}

	public bool PreFilterMessage(ref Message message)
	{
		if (message.Msg == 537)
		{
			int num = (int)message.WParam;
			if (num == 7)
			{
				bool flag = HasMouseAttached();
				if (flag != mouseAttached)
				{
					mouseAttached = flag;
					if (MouseStateChanged != null)
					{
						MouseStateChanged(mouseAttached);
					}
				}
			}
		}
		return false;
	}
}
