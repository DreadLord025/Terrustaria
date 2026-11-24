using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using ReLogic.Content.Sources;

namespace ReLogic.Content;

public class AssetRepository : IAssetRepository, IDisposable
{
	private readonly Dictionary<string, IAsset> _assets = new Dictionary<string, IAsset>();

	private IEnumerable<IContentSource> _sources = new IContentSource[0];

	private readonly Dictionary<Type, Action<IAsset, AssetRequestMode>> _typeSpecificReloadActions = new Dictionary<Type, Action<IAsset, AssetRequestMode>>();

	private readonly Dictionary<Type, Func<IAsset, IContentSource, string, Func<Stream>, bool>> _typeSpecificUpdateActions = new Dictionary<Type, Func<IAsset, IContentSource, string, Func<Stream>, bool>>();

	private readonly IAsyncAssetLoader _asyncLoader;

	private readonly IAssetLoader _loader;

	private AssetChangeWatcher _changeWatcher;

	private readonly object _requestLock = new object();

	private bool _isDisposed;

	public int PendingAssets => _asyncLoader.Remaining;

	public int TotalAssets { get; private set; }

	public int LoadedAssets { get; private set; }

	public bool IsAsyncLoadingEnabled => _asyncLoader.IsRunning;

	public AssetValueUpdated AssetValueUpdatedHandler { get; set; }

	public FailedToLoadAssetCustomAction AssetLoadFailHandler { get; set; }

	public AssetWatcherValueUpdated AssetWatcherValueUpdatedHandler { get; set; }

	public AssetWatcherUpdateFailed AssetWatcherUpdateFailedHandler { get; set; }

	public ContentFileUpdated ContentFileUpdatedHandler { get; set; }

	public AssetRepository(IAssetLoader syncLoader, IAsyncAssetLoader asyncLoader)
	{
		_loader = syncLoader;
		_asyncLoader = asyncLoader;
		_asyncLoader.Start();
	}

	internal AssetRepository(IAssetLoader syncLoader, IAsyncAssetLoader asyncLoader, bool useAsync)
	{
		_loader = syncLoader;
		_asyncLoader = asyncLoader;
		if (useAsync)
		{
			_asyncLoader.Start();
		}
	}

	public void SetSources(IEnumerable<IContentSource> sources, AssetRequestMode mode = AssetRequestMode.ImmediateLoad)
	{
		ThrowIfDisposed();
		lock (_requestLock)
		{
			while (_asyncLoader.Remaining > 0)
			{
				_asyncLoader.TransferCompleted();
			}
			_sources = sources.ToList();
			ReloadAssetsIfSourceChanged(mode);
		}
		if (_changeWatcher != null)
		{
			_changeWatcher.UpdateSources(sources);
		}
	}

	public Asset<T> Request<T>(string assetName, AssetRequestMode mode = AssetRequestMode.ImmediateLoad) where T : class
	{
		ThrowIfDisposed();
		assetName = AssetPathHelper.CleanPath(assetName);
		lock (_requestLock)
		{
			Asset<T> asset = null;
			if (_assets.TryGetValue(assetName, out var value))
			{
				asset = value as Asset<T>;
			}
			if (asset == null)
			{
				asset = new Asset<T>(assetName);
				_assets[assetName] = asset;
			}
			else if (asset.State != AssetState.NotLoaded)
			{
				return asset;
			}
			LoadAsset(asset, mode);
			return asset;
		}
	}

	public void TransferCompletedAssets()
	{
		ThrowIfDisposed();
		lock (_requestLock)
		{
			_asyncLoader.TransferCompleted();
		}
		UpdateWatchedAssetChanges();
	}

	public void EnableAssetWatcher()
	{
		ThrowIfDisposed();
		if (_changeWatcher == null)
		{
			_changeWatcher = new AssetChangeWatcher();
			_changeWatcher.UpdateSources(_sources);
		}
	}

	private void UpdateWatchedAssetChanges()
	{
		if (_changeWatcher == null)
		{
			return;
		}
		FileChangeWatcher<IContentSource>.FileUpdate[] updates = _changeWatcher.GetUpdates();
		for (int i = 0; i < updates.Length; i++)
		{
			FileChangeWatcher<IContentSource>.FileUpdate fileUpdate = updates[i];
			if (!UpdateWatchedAsset(fileUpdate.Source, fileUpdate.Path, fileUpdate.FullPath) && ContentFileUpdatedHandler != null)
			{
				ContentFileUpdatedHandler(fileUpdate.Source, fileUpdate.Path, fileUpdate.FullPath);
			}
		}
	}

	private bool UpdateWatchedAsset(IContentSource source, string path, string fullPath)
	{
		string extension = Path.GetExtension(path);
		string text = path.Substring(0, path.Length - extension.Length);
		lock (_requestLock)
		{
			if (!source.HasAsset(text))
			{
				source.Refresh();
			}
			if (!_assets.TryGetValue(text, out var value))
			{
				return false;
			}
			if (!value.IsLoaded)
			{
				return true;
			}
			_ = value.Source;
			if (source != value.Source && source != FindSourceForAsset(value.Name))
			{
				return true;
			}
			try
			{
				if (!UpdateAssetValue(value, source, extension, () => File.OpenRead(fullPath)))
				{
					return false;
				}
				if (AssetWatcherValueUpdatedHandler != null)
				{
					AssetWatcherValueUpdatedHandler(value);
				}
			}
			catch (Exception e)
			{
				if (AssetWatcherUpdateFailedHandler != null)
				{
					AssetWatcherUpdateFailedHandler(value, e);
				}
			}
			return true;
		}
	}

	private void ReloadAssetsIfSourceChanged(AssetRequestMode mode)
	{
		foreach (IAsset item in _assets.Values.Where((IAsset asset) => asset.IsLoaded))
		{
			IContentSource contentSource = FindSourceForAsset(item.Name);
			if (contentSource == null)
			{
				ForceReloadAsset(item, AssetRequestMode.DoNotLoad);
			}
			else if (item.Source != contentSource)
			{
				ForceReloadAsset(item, (item.State != AssetState.NotLoaded) ? mode : AssetRequestMode.DoNotLoad);
			}
		}
	}

	private void LoadAsset<T>(Asset<T> asset, AssetRequestMode mode) where T : class
	{
		if (mode == AssetRequestMode.DoNotLoad)
		{
			return;
		}
		EnsureTypeSpecificActionsExist<T>();
		TotalAssets++;
		asset.SetToLoadingState();
		try
		{
			TryLoadingAsset(asset, mode);
		}
		catch (Exception e)
		{
			if (AssetLoadFailHandler != null)
			{
				AssetLoadFailHandler(asset.Name, e);
			}
			throw;
		}
	}

	private void TryLoadingAsset<T>(Asset<T> asset, AssetRequestMode mode) where T : class
	{
		IContentSource source = FindSourceForAsset(asset.Name);
		switch (mode)
		{
		case AssetRequestMode.ImmediateLoad:
		{
			bool flag = true;
			IRejectionReason rejectionReason = null;
			T resultAsset;
			try
			{
				if (!_loader.TryLoad<T>(asset.Name, source, out resultAsset))
				{
					source.RejectAsset(asset.Name, new ContentRejectionFromFailedLoad_ByAssetExtensionType());
					TryLoadingAsset(asset, mode);
					break;
				}
			}
			catch (Exception ex)
			{
				bool flag2 = false;
				if (ex is UnauthorizedAccessException)
				{
					flag2 = true;
				}
				if (ex is FileNotFoundException)
				{
					flag2 = true;
				}
				if (ex is DirectoryNotFoundException)
				{
					flag2 = true;
				}
				if (ex is AssetLoadException)
				{
					flag2 = true;
				}
				if (flag2)
				{
					throw;
				}
				throw AssetLoadException.FromAssetException(asset.Name, ex);
			}
			if (source.ContentValidator != null)
			{
				flag = source.ContentValidator.AssetIsValid(resultAsset, asset.Name, out rejectionReason);
			}
			if (flag)
			{
				SubmitLoadedContent(asset, resultAsset, source);
				LoadedAssets++;
			}
			else
			{
				source.RejectAsset(asset.Name, rejectionReason);
				TryLoadingAsset(asset, mode);
			}
			break;
		}
		case AssetRequestMode.AsyncLoad:
			_asyncLoader.Load(asset.Name, source, (bool proper, T content) =>
			{
				if (!proper)
				{
					source.RejectAsset(asset.Name, new ContentRejectionFromFailedLoad_ByAssetExtensionType());
					TryLoadingAsset(asset, mode);
				}
				else
				{
					bool flag3 = true;
					IRejectionReason rejectionReason2 = null;
					if (source.ContentValidator != null)
					{
						flag3 = source.ContentValidator.AssetIsValid(content, asset.Name, out rejectionReason2);
					}
					if (flag3)
					{
						SubmitLoadedContent(asset, content, source);
						int loadedAssets = LoadedAssets;
						LoadedAssets = loadedAssets + 1;
					}
					else
					{
						source.RejectAsset(asset.Name, rejectionReason2);
						TryLoadingAsset(asset, mode);
					}
				}
			});
			break;
		default:
			throw new ArgumentOutOfRangeException("mode", mode, null);
		}
	}

	private void ForceReloadAsset(IAsset asset, AssetRequestMode mode)
	{
		if (mode != AssetRequestMode.DoNotLoad)
		{
			LoadedAssets--;
		}
		_typeSpecificReloadActions[asset.GetType()](asset, mode);
	}

	private void EnsureTypeSpecificActionsExist<T>() where T : class
	{
		_typeSpecificReloadActions[typeof(Asset<T>)] = ForceReloadAsset<T>;
		_typeSpecificUpdateActions[typeof(Asset<T>)] = UpdateAssetValue<T>;
	}

	private void ForceReloadAsset<T>(IAsset asset, AssetRequestMode mode) where T : class
	{
		Asset<T> asset2 = (Asset<T>)asset;
		asset2.Unload();
		LoadAsset(asset2, mode);
	}

	private bool UpdateAssetValue(IAsset asset, IContentSource source, string extension, Func<Stream> getStream)
	{
		return _typeSpecificUpdateActions[asset.GetType()](asset, source, extension, getStream);
	}

	private bool UpdateAssetValue<T>(IAsset asset, IContentSource source, string extension, Func<Stream> getStream) where T : class
	{
		if (!_loader.TryLoad<T>(extension, getStream, out var resultAsset))
		{
			return false;
		}
		IContentValidator contentValidator = source.ContentValidator;
		if (contentValidator != null && !contentValidator.AssetIsValid(resultAsset, asset.Name, out var rejectionReason))
		{
			throw AssetLoadException.RejectedByValidator(rejectionReason);
		}
		SubmitLoadedContent((Asset<T>)asset, resultAsset, source);
		return true;
	}

	private void SubmitLoadedContent<T>(Asset<T> asset, T value, IContentSource source) where T : class
	{
		asset.SubmitLoadedContent(value, source);
		if (AssetValueUpdatedHandler != null)
		{
			AssetValueUpdatedHandler(asset, value);
		}
	}

	private IContentSource FindSourceForAsset(string assetName)
	{
		return _sources.FirstOrDefault((IContentSource source) => source.HasAsset(assetName)) ?? throw AssetLoadException.FromMissingAsset(assetName);
	}

	private void ThrowIfDisposed()
	{
		if (_isDisposed)
		{
			throw new ObjectDisposedException("AssetRepository is disposed.");
		}
	}

	protected virtual void Dispose(bool disposing)
	{
		if (_isDisposed)
		{
			return;
		}
		if (disposing)
		{
			_asyncLoader.Dispose();
			foreach (KeyValuePair<string, IAsset> asset in _assets)
			{
				asset.Value.Dispose();
			}
			if (_changeWatcher != null)
			{
				_changeWatcher.Dispose();
			}
		}
		_isDisposed = true;
	}

	public void Dispose()
	{
		Dispose(disposing: true);
	}
}
