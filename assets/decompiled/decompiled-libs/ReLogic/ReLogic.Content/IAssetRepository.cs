using System;
using System.Collections.Generic;
using ReLogic.Content.Sources;

namespace ReLogic.Content;

public interface IAssetRepository : IDisposable
{
	int PendingAssets { get; }

	int TotalAssets { get; }

	int LoadedAssets { get; }

	AssetValueUpdated AssetValueUpdatedHandler { get; set; }

	FailedToLoadAssetCustomAction AssetLoadFailHandler { get; set; }

	AssetWatcherValueUpdated AssetWatcherValueUpdatedHandler { get; set; }

	AssetWatcherUpdateFailed AssetWatcherUpdateFailedHandler { get; set; }

	ContentFileUpdated ContentFileUpdatedHandler { get; set; }

	void SetSources(IEnumerable<IContentSource> sources, AssetRequestMode mode = AssetRequestMode.ImmediateLoad);

	Asset<T> Request<T>(string assetName, AssetRequestMode mode = AssetRequestMode.ImmediateLoad) where T : class;

	void TransferCompletedAssets();

	void EnableAssetWatcher();
}
