using System;
using System.Collections.Generic;
using System.IO;

namespace ReLogic.Content.Sources;

public class XnaDirectContentSource : IContentSource
{
	private readonly string _rootDirectory;

	private readonly HashSet<string> _files = new HashSet<string>();

	private readonly RejectedAssetCollection _rejections = new RejectedAssetCollection();

	public IContentValidator ContentValidator { get; set; }

	public string FileWatcherPath => null;

	public XnaDirectContentSource(string rootDirectory)
	{
		_rootDirectory = AssetPathHelper.CleanPath(rootDirectory);
		Refresh();
	}

	public void Refresh()
	{
		_files.Clear();
		string[] files = Directory.GetFiles(_rootDirectory, "*.xnb", SearchOption.AllDirectories);
		foreach (string text in files)
		{
			_files.Add(text.ToLower());
		}
	}

	public bool HasAsset(string assetName)
	{
		string text = Path.Combine(_rootDirectory, assetName) + ".xnb";
		if (_rejections.IsRejected(assetName))
		{
			return false;
		}
		return _files.Contains(text.ToLower());
	}

	public List<string> GetAllAssetsStartingWith(string assetNameStart)
	{
		string value = Path.Combine(_rootDirectory, assetNameStart).ToLower();
		List<string> list = new List<string>();
		foreach (string file in _files)
		{
			if (file.StartsWith(value))
			{
				list.Add(file);
			}
		}
		return list;
	}

	public string GetExtension(string assetName)
	{
		return ".xnb";
	}

	public Stream OpenStream(string assetName)
	{
		string path = Path.Combine(_rootDirectory, assetName) + ".xnb";
		try
		{
			return File.OpenRead(path);
		}
		catch (Exception innerException)
		{
			throw AssetLoadException.FromMissingAsset(assetName, innerException);
		}
	}

	public void RejectAsset(string assetName, IRejectionReason reason)
	{
		_rejections.Reject(assetName, reason);
	}

	public void ClearRejections()
	{
		_rejections.Clear();
	}

	public bool TryGetRejections(List<string> rejectionReasons)
	{
		return _rejections.TryGetRejections(rejectionReasons);
	}
}
