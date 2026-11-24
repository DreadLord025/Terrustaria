using System;
using ReLogic.Content.Sources;

namespace ReLogic.Content;

public interface IAsyncAssetLoader : IDisposable
{
	bool IsRunning { get; }

	int Remaining { get; }

	void Start();

	void Stop();

	void Load<T>(string assetName, IContentSource source, LoadAssetDelegate<T> callback) where T : class;

	void TransferCompleted();
}
