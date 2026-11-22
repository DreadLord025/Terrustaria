using System;
using System.Collections.Generic;
using System.Diagnostics.CodeAnalysis;
using System.IO;
using System.Linq;

namespace ReLogic.Content;

[SuppressMessage("Microsoft.Design", "CA1063:ImplementIDisposableCorrectly", Justification = "Class only contains managed resources")]
public class FileChangeWatcher<T> : IDisposable
{
	public struct FileUpdate : IEquatable<FileUpdate>
	{
		public T Source;

		public string Path;

		public string FullPath;

		public override int GetHashCode()
		{
			return Source.GetHashCode() ^ Path.GetHashCode();
		}

		public bool Equals(FileUpdate other)
		{
			return this == other;
		}

		public override bool Equals(object obj)
		{
			if (obj is FileUpdate)
			{
				return Equals((FileUpdate)obj);
			}
			return false;
		}

		public static bool operator ==(FileUpdate a, FileUpdate b)
		{
			if (EqualityComparer<T>.Default.Equals(a.Source, b.Source))
			{
				return a.Path == b.Path;
			}
			return false;
		}

		public static bool operator !=(FileUpdate a, FileUpdate b)
		{
			return !(a == b);
		}
	}

	private const int UPDATE_COOLDOWN = 10;

	private readonly List<FileSystemWatcher> _watchers = new List<FileSystemWatcher>();

	private readonly HashSet<FileUpdate> _updates = new HashSet<FileUpdate>();

	private int _cooldown;

	protected bool _disposed;

	private static FileUpdate[] _empty = new FileUpdate[0];

	public FileSystemWatcher NewWatcher(T source, string path)
	{
		FileSystemWatcher fileSystemWatcher = new FileSystemWatcher();
		fileSystemWatcher.Path = path;
		fileSystemWatcher.NotifyFilter = NotifyFilters.FileName | NotifyFilters.DirectoryName | NotifyFilters.LastWrite;
		fileSystemWatcher.IncludeSubdirectories = true;
		fileSystemWatcher.Changed += (object sender, FileSystemEventArgs e) =>
		{
			AddFileUpdate(source, e);
		};
		fileSystemWatcher.Created += (object sender, FileSystemEventArgs e) =>
		{
			AddFileOrDirectoryUpdateRecursive(source, e);
		};
		fileSystemWatcher.Renamed += (object sender, RenamedEventArgs e) =>
		{
			AddFileOrDirectoryUpdateRecursive(source, e);
		};
		fileSystemWatcher.EnableRaisingEvents = true;
		return fileSystemWatcher;
	}

	private void AddFileUpdate(T source, FileSystemEventArgs e)
	{
		if (!Directory.Exists(e.FullPath))
		{
			AddUpdate(source, e);
		}
	}

	private void AddFileOrDirectoryUpdateRecursive(T source, FileSystemEventArgs e)
	{
		if (!Directory.Exists(e.FullPath))
		{
			AddUpdate(source, e);
			return;
		}
		string text = e.FullPath.Substring(0, e.FullPath.Length - e.Name.Length);
		foreach (string item in Directory.EnumerateFiles(e.FullPath, "*", SearchOption.AllDirectories))
		{
			AddUpdate(source, new FileSystemEventArgs(e.ChangeType, text, item.Substring(text.Length)));
		}
	}

	private void AddUpdate(T source, FileSystemEventArgs e)
	{
		lock (_updates)
		{
			_updates.Add(new FileUpdate
			{
				Source = source,
				Path = e.Name,
				FullPath = e.FullPath
			});
			_cooldown = 10;
		}
	}

	public FileUpdate[] GetUpdates()
	{
		lock (_updates)
		{
			if (_cooldown <= 0 || --_cooldown > 0)
			{
				return _empty;
			}
			FileUpdate[] result = _updates.ToArray();
			_updates.Clear();
			return result;
		}
	}

	[SuppressMessage("Microsoft.Design", "CA1063:ImplementIDisposableCorrectly", Justification = "Class only contains managed resources")]
	public void Dispose()
	{
		if (_disposed)
		{
			return;
		}
		foreach (FileSystemWatcher watcher in _watchers)
		{
			watcher.Dispose();
		}
		_watchers.Clear();
		_disposed = true;
	}
}
