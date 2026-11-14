// S1-L-5 fixture generator for the .NET Framework runtime (the vanilla one).
//
// Terraria 1.4.5.8 runs on .NET Framework 4.8(.1), whose `DeflateStream`
// interops the machine's native zlib through `ZLibNative` — a different
// implementation (and possibly different zlib version) than the .NET Core/10
// one, so the parity fixture must come from THIS runtime. Compile and run:
//   csc /nologo /r:System.IO.Compression.dll -out:MapDeflateFw48.exe MapDeflateFw48.cs
//   MapDeflateFw48.exe <input-body> <output-deflated>
// (System.IO.Compression.dll resolves from the GAC; see the repo notes.)
//
// Mirrors `MapHelper.InternalSaveMap` (`decompiled/Terraria/Map/MapHelper.cs:3030-3322`):
// a raw `DeflateStream` (no zlib/gzip wrapper) at the `CompressionMode.Compress`
// ctor default (`CompressionLevel.Optimal`), the body staged in a 16384-byte
// array flushed to the stream in at most 4096-byte chunks. Also compresses in a
// single write and reports whether the two agree.
using System;
using System.IO;
using System.IO.Compression;
using System.Security.Cryptography;

static class MapDeflateFw48
{
    static byte[] CompressChunked(byte[] body)
    {
        using (var memoryStream = new MemoryStream(4000))
        using (var deflateStream = new DeflateStream(memoryStream, CompressionMode.Compress))
        {
            for (int offset = 0; offset < body.Length; offset += 4096)
            {
                int num = Math.Min(4096, body.Length - offset);
                deflateStream.Write(body, offset, num);
            }
            deflateStream.Dispose();
            return memoryStream.ToArray();
        }
    }

    static byte[] CompressSingle(byte[] body)
    {
        using (var memoryStream = new MemoryStream(4000))
        using (var deflateStream = new DeflateStream(memoryStream, CompressionMode.Compress))
        {
            deflateStream.Write(body, 0, body.Length);
            deflateStream.Dispose();
            return memoryStream.ToArray();
        }
    }

    static string Sha(byte[] data)
    {
        using (var sha = SHA256.Create())
            return BitConverter.ToString(sha.ComputeHash(data)).Replace("-", "");
    }

    static byte[] CompressLevel(byte[] body, CompressionLevel level)
    {
        using (var memoryStream = new MemoryStream(4000))
        using (var deflateStream = new DeflateStream(memoryStream, level))
        {
            deflateStream.Write(body, 0, body.Length);
            deflateStream.Dispose();
            return memoryStream.ToArray();
        }
    }

    static int Main(string[] args)
    {
        if (args.Length != 2)
        {
            Console.Error.WriteLine("usage: MapDeflateFw48 <input-body> <output-deflated>");
            return 1;
        }
        byte[] body = File.ReadAllBytes(args[0]);
        byte[] chunked = CompressChunked(body);
        byte[] single = CompressSingle(body);
        bool equal = chunked.Length == single.Length;
        if (equal)
        {
            for (int i = 0; i < chunked.Length && equal; i++)
                equal = chunked[i] == single[i];
        }
        Console.WriteLine("chunked == single: " + (equal ? "yes" : "NO"));
        File.WriteAllBytes(args[1], chunked);
        Console.WriteLine("runtime: " + Environment.Version);
        Console.WriteLine(string.Format("input  {0,8} bytes  sha256 {1}", body.Length, Sha(body)));
        Console.WriteLine(string.Format("output {0,8} bytes  sha256 {1}", chunked.Length, Sha(chunked)));
        // Which level does the CompressionMode default actually compress like?
        foreach (CompressionLevel level in new[]
        {
            CompressionLevel.NoCompression, CompressionLevel.Fastest,
            CompressionLevel.Optimal,
        })
        {
            byte[] at = CompressLevel(body, level);
            bool isChunked = at.Length == chunked.Length && BitConverter.ToString(at) == BitConverter.ToString(chunked);
            Console.WriteLine(string.Format("level {0,-12} -> {1,8} bytes {2}", level, at.Length, isChunked ? "(== CompressionMode output)" : ""));
        }
        return 0;
    }
}
