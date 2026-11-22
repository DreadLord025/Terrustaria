using System;
using ReLogic.Content.Sources;

namespace ReLogic.Content;

public sealed class Asset<T> : IAsset, IDisposable where T : class
{
	public static readonly Asset<T> Empty = new Asset<T>("");

	public string Name { get; private set; }

	public bool IsLoaded => State == AssetState.Loaded;

	public AssetState State { get; private set; }

	public bool IsDisposed { get; private set; }

	public IContentSource Source { get; private set; }

	public T Value { get; private set; }

	internal Asset(string name)
	{
		State = AssetState.NotLoaded;
		Name = name;
	}

	public static explicit operator T(Asset<T> asset)
	{
		return asset.Value;
	}

	internal void Unload()
	{
		if (Value is IDisposable disposable)
		{
			disposable.Dispose();
		}
		State = AssetState.NotLoaded;
		Value = null;
		Source = null;
	}

	internal void SubmitLoadedContent(T value, IContentSource source)
	{
		if (value == null)
		{
			throw new ArgumentNullException("value");
		}
		if (source == null)
		{
			throw new ArgumentNullException("source");
		}
		if (Value is IDisposable disposable)
		{
			disposable.Dispose();
		}
		State = AssetState.Loaded;
		Value = value;
		Source = source;
	}

	internal void SetToLoadingState()
	{
		State = AssetState.Loading;
	}

	private void Dispose(bool disposing)
	{
		if (IsDisposed)
		{
			return;
		}
		if (disposing && Value != null)
		{
			IDisposable disposable = Value as IDisposable;
			if (IsLoaded)
			{
				disposable?.Dispose();
			}
			Value = null;
		}
		IsDisposed = true;
	}

	public void Dispose()
	{
		Dispose(disposing: true);
	}
}
