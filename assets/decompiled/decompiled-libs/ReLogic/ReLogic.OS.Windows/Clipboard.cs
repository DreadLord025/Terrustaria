using System;
using System.Threading;
using System.Windows.Forms;
using ReLogic.OS.Base;

namespace ReLogic.OS.Windows;

internal class Clipboard : ReLogic.OS.Base.Clipboard
{
	protected override string GetClipboard()
	{
		return TryToGetClipboardText();
	}

	protected override void SetClipboard(string text)
	{
		if (string.IsNullOrEmpty(text))
		{
			return;
		}
		try
		{
			InvokeInStaThread(() =>
			{
				System.Windows.Forms.Clipboard.SetText(text);
			});
		}
		catch
		{
			Console.WriteLine("Failed to set clipboard contents!");
		}
	}

	private string TryToGetClipboardText()
	{
		string result;
		try
		{
			result = InvokeInStaThread(() => System.Windows.Forms.Clipboard.GetText());
		}
		catch
		{
			result = "";
			Console.WriteLine("Failed to get clipboard contents!");
		}
		return result;
	}

	private static T InvokeInStaThread<T>(Func<T> callback)
	{
		if (GetApartmentStateSafely() == ApartmentState.STA)
		{
			return callback();
		}
		T result = default;
		Thread thread = new Thread(() =>
		{
			result = callback();
		});
		thread.SetApartmentState(ApartmentState.STA);
		thread.Start();
		thread.Join();
		return result;
	}

	private static void InvokeInStaThread(Action callback)
	{
		if (GetApartmentStateSafely() == ApartmentState.STA)
		{
			callback();
			return;
		}
		Thread thread = new Thread(() =>
		{
			callback();
		});
		thread.SetApartmentState(ApartmentState.STA);
		thread.Start();
		thread.Join();
	}

	private static ApartmentState GetApartmentStateSafely()
	{
		try
		{
			return Thread.CurrentThread.GetApartmentState();
		}
		catch
		{
			return ApartmentState.MTA;
		}
	}
}
