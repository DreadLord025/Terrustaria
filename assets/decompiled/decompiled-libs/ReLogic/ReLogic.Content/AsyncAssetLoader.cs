using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.IO;
using ReLogic.Content.Sources;
using ReLogic.Threading;

namespace ReLogic.Content;

public class AsyncAssetLoader : IAsyncAssetLoader, IDisposable
{
	private readonly AsyncActionDispatcher _loadDispatcher = new AsyncActionDispatcher();

	private readonly AssetReaderCollection _readers;

	private readonly ConcurrentQueue<Action> _pendingCallbacks = new ConcurrentQueue<Action>();

	private readonly ConcurrentQueue<Action> _pendingDelayedCallbacks = new ConcurrentQueue<Action>();

	private readonly HashSet<Type> _delayedLoadTypes = new HashSet<Type>();

	private readonly int _maxCreationsPerTransfer;

	private bool _isDisposed;

	public bool IsRunning => _loadDispatcher.IsRunning;

	public int Remaining { get; private set; }

	internal int AssetsReadyForTransfer => _pendingCallbacks.Count + _pendingDelayedCallbacks.Count;

	public AsyncAssetLoader(AssetReaderCollection readers)
	{
		_readers = readers;
		_maxCreationsPerTransfer = int.MaxValue;
	}

	public AsyncAssetLoader(AssetReaderCollection readers, int maxCreationsPerTransfer)
	{
		_readers = readers;
		_maxCreationsPerTransfer = maxCreationsPerTransfer;
	}

	public void Load<T>(string assetName, IContentSource source, LoadAssetDelegate<T> callback) where T : class
	{
		Remaining++;
		if (_delayedLoadTypes.Contains(typeof(T)))
		{
			DelayedLoad(assetName, source, callback);
		}
		else
		{
			FullLoad(assetName, source, callback);
		}
	}

	public void RequireTypeCreationOnTransfer(Type type)
	{
		_delayedLoadTypes.Add(type);
	}

	public void TransferCompleted()
	{
		Action result;
		while (_pendingCallbacks.TryDequeue(out result))
		{
			result();
			Remaining--;
		}
		for (int i = 0; i < _maxCreationsPerTransfer; i++)
		{
			if (!_pendingDelayedCallbacks.TryDequeue(out result))
			{
				break;
			}
			result();
			Remaining--;
		}
	}

	public void Start()
	{
		_loadDispatcher.Start();
	}

	public void Stop()
	{
		_loadDispatcher.Stop();
	}

	private void FullLoad<T>(string assetName, IContentSource source, LoadAssetDelegate<T> callback) where T : class
	{
		_loadDispatcher.Queue(() =>
		{
			string extension = source.GetExtension(assetName);
			T resultAsset = null;
			if (_readers.CanReadExtension(extension))
			{
				resultAsset = _readers.Read<T>(source.OpenStream(assetName), extension);
			}
			_pendingCallbacks.Enqueue(() =>
			{
				callback(resultAsset != null, resultAsset);
			});
		});
	}

	private void DelayedLoad<T>(string assetName, IContentSource source, LoadAssetDelegate<T> callback) where T : class
	{
		_loadDispatcher.Queue(() =>
		{
			string extension = source.GetExtension(assetName);
			if (!_readers.CanReadExtension(extension))
			{
				_pendingDelayedCallbacks.Enqueue(() =>
				{
					callback(loadedSuccessfully: false, null);
				});
			}
			else
			{
				byte[] rawContent;
				using (Stream stream = source.OpenStream(assetName))
				{
					rawContent = new byte[stream.Length];
					stream.Read(rawContent, 0, rawContent.Length);
				}
				_pendingDelayedCallbacks.Enqueue(() =>
				{
					T theAsset;
					using (MemoryStream stream2 = new MemoryStream(rawContent, 0, rawContent.Length, writable: false, publiclyVisible: true))
					{
						theAsset = _readers.Read<T>(stream2, extension);
					}
					callback(loadedSuccessfully: true, theAsset);
				});
			}
		});
	}

	protected virtual void Dispose(bool disposing)
	{
		if (!_isDisposed)
		{
			if (disposing)
			{
				Stop();
				_loadDispatcher.Dispose();
			}
			_isDisposed = true;
		}
	}

	public void Dispose()
	{
		Dispose(disposing: true);
	}
}
