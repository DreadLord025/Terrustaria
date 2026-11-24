using System;
using System.IO;
using System.Threading;
using Microsoft.Xna.Framework.Content;

namespace ReLogic.Content.Readers;

public class XnbReader : IAssetReader, IDisposable
{
	private class InternalContentManager : ContentManager
	{
		private Stream _stream;

		public InternalContentManager(IServiceProvider serviceProvider)
			: base(serviceProvider)
		{
		}

		public void SetStream(Stream stream)
		{
			_stream = stream;
		}

		public T Load<T>()
		{
			return ReadAsset<T>("XnaAsset", null);
		}

		protected override Stream OpenStream(string assetName)
		{
			return _stream;
		}
	}

	private readonly ThreadLocal<InternalContentManager> _contentLoader;

	private bool _disposedValue;

	public XnbReader(IServiceProvider services)
	{
		_contentLoader = new ThreadLocal<InternalContentManager>(() => new InternalContentManager(services));
	}

	public T FromStream<T>(Stream stream) where T : class
	{
		InternalContentManager value = _contentLoader.Value;
		value.SetStream(stream);
		return value.Load<T>();
	}

	protected virtual void Dispose(bool disposing)
	{
		if (!_disposedValue)
		{
			if (disposing)
			{
				_contentLoader.Dispose();
			}
			_disposedValue = true;
		}
	}

	public void Dispose()
	{
		Dispose(disposing: true);
	}
}
