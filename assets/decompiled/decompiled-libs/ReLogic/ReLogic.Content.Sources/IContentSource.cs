using System.Collections.Generic;
using System.IO;

namespace ReLogic.Content.Sources;

public interface IContentSource
{
	IContentValidator ContentValidator { get; set; }

	string FileWatcherPath { get; }

	bool HasAsset(string assetName);

	List<string> GetAllAssetsStartingWith(string assetNameStart);

	string GetExtension(string assetName);

	Stream OpenStream(string assetName);

	void RejectAsset(string assetName, IRejectionReason reason);

	void ClearRejections();

	bool TryGetRejections(List<string> rejectionReasons);

	void Refresh();
}
