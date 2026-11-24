using System;
using System.IO;
using System.Threading;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Graphics;

namespace ReLogic.Content.Readers;

public class PngReader : IAssetReader, IDisposable
{
	private readonly GraphicsDevice _graphicsDevice;

	private readonly ThreadLocal<Color[]> _colorProcessingCache;

	private bool _disposedValue;

	public PngReader(GraphicsDevice graphicsDevice)
	{
		_graphicsDevice = graphicsDevice;
		_colorProcessingCache = new ThreadLocal<Color[]>();
	}

	public T FromStream<T>(Stream stream) where T : class
	{
		if (typeof(T) != typeof(Texture2D))
		{
			throw AssetLoadException.FromInvalidReader<PngReader, T>();
		}
		Texture2D texture2D = Texture2D.FromStream(_graphicsDevice, stream);
		int num = texture2D.Width * texture2D.Height;
		if (!_colorProcessingCache.IsValueCreated || _colorProcessingCache.Value.Length < num)
		{
			_colorProcessingCache.Value = new Color[num];
		}
		Color[] value = _colorProcessingCache.Value;
		texture2D.GetData(value, 0, num);
		for (int i = 0; i != num; i++)
		{
			value[i] = Color.FromNonPremultiplied(value[i].ToVector4());
		}
		texture2D.SetData(value, 0, num);
		return texture2D as T;
	}

	protected virtual void Dispose(bool disposing)
	{
		if (!_disposedValue)
		{
			if (disposing)
			{
				_colorProcessingCache.Dispose();
			}
			_disposedValue = true;
		}
	}

	public void Dispose()
	{
		Dispose(disposing: true);
	}
}
