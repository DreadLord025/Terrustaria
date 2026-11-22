using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using ReLogic.Content.Sources;

namespace ReLogic.Content;

public sealed class AssetChangeWatcher : FileChangeWatcher<IContentSource>
{
	private readonly Dictionary<FileSystemWatcher, IContentSource> _watchers = new Dictionary<FileSystemWatcher, IContentSource>();

	public void UpdateSources(IEnumerable<IContentSource> sources)
	{
		lock (_watchers)
		{
			if (_disposed)
			{
				throw new ObjectDisposedException("AssetFileWatchers");
			}
			List<FileSystemWatcher> list = _watchers.Keys.ToList();
			_watchers.Clear();
			foreach (IContentSource source in sources)
			{
				string path = source.FileWatcherPath;
				if (path != null && Directory.Exists(path))
				{
					FileSystemWatcher fileSystemWatcher = list.FirstOrDefault((FileSystemWatcher e) => e.Path == path);
					if (fileSystemWatcher == null)
					{
						fileSystemWatcher = NewWatcher(source, path);
					}
					_watchers[fileSystemWatcher] = source;
				}
			}
			foreach (FileSystemWatcher item in list)
			{
				if (!_watchers.ContainsKey(item))
				{
					item.Dispose();
				}
			}
		}
	}
}
