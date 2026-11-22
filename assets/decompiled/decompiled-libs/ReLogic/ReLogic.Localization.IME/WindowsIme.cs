using System;
using System.Runtime.InteropServices;
using System.Windows.Forms;
using ReLogic.Localization.IME.Windows;
using ReLogic.OS.Windows;

namespace ReLogic.Localization.IME;

internal class WindowsIme : PlatformIme, IMessageFilter
{
	private IntPtr _windowHandle;

	private bool _isFocused;

	private bool _visibleUnfocusedCandidateWindow;

	private WindowsMessageHook _wndProcHook;

	private bool _disposedValue;

	public override string CompositionString => Marshal.PtrToStringUni(ReLogic.Localization.IME.Windows.NativeMethods.ImeUi_GetCompositionString()).ToString();

	public override bool IsCandidateListVisible => ReLogic.Localization.IME.Windows.NativeMethods.ImeUi_IsCandidateListVisible();

	public override uint? SelectedCandidate
	{
		get
		{
			if (!_visibleUnfocusedCandidateWindow)
			{
				return ReLogic.Localization.IME.Windows.NativeMethods.ImeUi_GetCandidateSelection();
			}
			return null;
		}
	}

	public override uint CandidateCount => ReLogic.Localization.IME.Windows.NativeMethods.ImeUi_GetCandidatePageSize();

	public WindowsIme(WindowsMessageHook wndProcHook, IntPtr windowHandle)
	{
		_wndProcHook = wndProcHook;
		_windowHandle = windowHandle;
		_isFocused = ReLogic.OS.Windows.NativeMethods.GetForegroundWindow() == _windowHandle;
		_wndProcHook.AddMessageFilter(this);
		ReLogic.Localization.IME.Windows.NativeMethods.ImeUi_Initialize(_windowHandle);
	}

	protected override void OnEnable()
	{
		if (_isFocused)
		{
			ReLogic.Localization.IME.Windows.NativeMethods.ImeUi_Enable(bEnable: true);
		}
	}

	protected override void OnDisable()
	{
		ReLogic.Localization.IME.Windows.NativeMethods.ImeUi_FinalizeString();
		ReLogic.Localization.IME.Windows.NativeMethods.ImeUi_Enable(bEnable: false);
		_visibleUnfocusedCandidateWindow = false;
	}

	public override string GetCandidate(uint index)
	{
		return Marshal.PtrToStringUni(ReLogic.Localization.IME.Windows.NativeMethods.ImeUi_GetCandidate(index));
	}

	public bool PreFilterMessage(ref Message message)
	{
		if (message.Msg == 8)
		{
			ReLogic.Localization.IME.Windows.NativeMethods.ImeUi_Enable(bEnable: false);
			_isFocused = false;
			_visibleUnfocusedCandidateWindow = false;
			return true;
		}
		if (message.Msg == 7)
		{
			if (IsEnabled)
			{
				ReLogic.Localization.IME.Windows.NativeMethods.ImeUi_Enable(bEnable: true);
			}
			_isFocused = true;
			return true;
		}
		if (!IsEnabled)
		{
			return false;
		}
		bool trapped = false;
		IntPtr lParam = message.LParam;
		ReLogic.Localization.IME.Windows.NativeMethods.ImeUi_ProcessMessage(message.HWnd, message.Msg, message.WParam, ref lParam, ref trapped);
		message.LParam = lParam;
		if (trapped)
		{
			return true;
		}
		switch (message.Msg)
		{
		case 81:
			return true;
		case 641:
			message.LParam = IntPtr.Zero;
			break;
		case 269:
			return true;
		case 642:
		{
			int num4 = message.WParam.ToInt32();
			if (num4 == 5 && ReLogic.Localization.IME.Windows.NativeMethods.ImeUi_GetPrimaryLanguage() == 17)
			{
				_visibleUnfocusedCandidateWindow = true;
			}
			else if (num4 == 4)
			{
				_visibleUnfocusedCandidateWindow = false;
			}
			return true;
		}
		case 258:
			OnKeyPress((char)message.WParam.ToInt32());
			break;
		case 256:
		{
			bool flag = false;
			uint? num = null;
			if ((int)message.WParam == 229 && ReLogic.Localization.IME.Windows.NativeMethods.ImeUi_GetPrimaryLanguage() == 17)
			{
				if (string.IsNullOrEmpty(CompositionString))
				{
					_visibleUnfocusedCandidateWindow = false;
				}
				else if (!_visibleUnfocusedCandidateWindow)
				{
					uint num2 = ReLogic.Localization.IME.Windows.NativeMethods.ImmGetVirtualKey(message.HWnd);
					if (num2 == 8 || num2 == 27)
					{
						_visibleUnfocusedCandidateWindow = true;
					}
					if (num2 == 9)
					{
						num = SelectedCandidate;
					}
				}
				else
				{
					uint num3 = ReLogic.Localization.IME.Windows.NativeMethods.ImmGetVirtualKey(message.HWnd);
					if (num3 == 40 || num3 == 9)
					{
						_visibleUnfocusedCandidateWindow = false;
						flag = SelectedCandidate == 0;
					}
				}
			}
			if (!ReLogic.Localization.IME.Windows.NativeMethods.ImeUi_ShouldIgnoreHotKey(ref message))
			{
				ReLogic.OS.Windows.NativeMethods.TranslateMessage(ref message);
			}
			if (flag && SelectedCandidate == 1)
			{
				ReLogic.Localization.IME.Windows.NativeMethods.ImeUi_SetCandidateSelection(0u);
			}
			if (num.HasValue && SelectedCandidate == num)
			{
				ReLogic.Localization.IME.Windows.NativeMethods.ImeUi_SetCandidateSelection(0u);
			}
			break;
		}
		}
		return false;
	}

	protected override void Dispose(bool disposing)
	{
		if (!_disposedValue)
		{
			if (IsEnabled)
			{
				Disable();
			}
			_wndProcHook.RemoveMessageFilter(this);
			ReLogic.Localization.IME.Windows.NativeMethods.ImeUi_Uninitialize();
			_disposedValue = true;
		}
	}

	~WindowsIme()
	{
		Dispose(disposing: false);
	}
}
