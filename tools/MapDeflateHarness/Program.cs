// S1-L-5 `.map` deflate-parity evidence generator.
//
// Terraria's `MapHelper.InternalSaveMap` (`decompiled/Terraria/Map/MapHelper.cs:3030-3322`)
// compresses the `.map` row body with a raw `DeflateStream` — no zlib/gzip
// wrapper — at the `CompressionMode.Compress` ctor default
// (`CompressionLevel.Optimal`, zlib level 6), staging the row bytes in a
// 16384-byte array that is flushed to the stream in 4096-byte chunks
// (`MapHelper.cs:3299-3306`).
//
// The vanilla runtime is .NET Framework 4.8(.1), whose `clrcompression.dll`
// embeds a Microsoft-patched zlib (version string 1.3.1, output not reproducible
// by any stock zlib — see `fw48/` and ANALYSIS.md, "The `.map` deflate stream").
// This harness documents that side: it compresses the frozen body fixture
// (`map-body.bin`) and, together with the `fw48/` csc.exe generator, produced
// the committed `fw48-reference.deflate`. The Rust `s1_l_5` test asserts the
// reference inflates back onto the same body and pins its own (zlib-rs, level 6)
// bytes separately.
//
// Usage:
//   dotnet run --project tools/MapDeflateHarness --
//       tools/MapDeflateHarness/map-body.bin <out> [single-write-out]
using System.IO.Compression;
using System.Security.Cryptography;

if (args.Length is < 2 or > 3)
{
    Console.Error.WriteLine("usage: MapDeflateHarness <input-body> <output-deflated> [single-write-output]");
    return 1;
}

byte[] body = File.ReadAllBytes(args[0]);

// The C# path: staged writes of at most 4096 bytes (last chunk = the remainder).
byte[] Chunked()
{
    using MemoryStream memoryStream = new(4000);
    using DeflateStream deflateStream = new(memoryStream, CompressionMode.Compress);
    for (int offset = 0; offset < body.Length; offset += 4096)
    {
        int num = Math.Min(4096, body.Length - offset);
        deflateStream.Write(body, offset, num);
    }
    deflateStream.Dispose();
    return memoryStream.ToArray();
}

// Same input, one write: .NET FW's zlib is chunk-invariant (the `fw48/`
// generator reports "chunked == single: yes"); a modern .NET's is not, so the
// extra output exists to make that difference visible.
byte[] chunked = Chunked();
byte[] single;
using (MemoryStream memoryStream = new(4000))
{
    using DeflateStream deflateStream = new(memoryStream, CompressionMode.Compress);
    deflateStream.Write(body, 0, body.Length);
    deflateStream.Dispose();
    single = memoryStream.ToArray();
}

if (!chunked.AsSpan().SequenceEqual(single))
{
    Console.Error.WriteLine("note: chunked and single-write deflate disagree on this runtime");
    if (args.Length == 3)
    {
        File.WriteAllBytes(args[2], single);
    }
}

File.WriteAllBytes(args[1], chunked);
Console.WriteLine($"runtime {Environment.Version}");
Console.WriteLine($"input  {body.Length,8} bytes  sha256 {Convert.ToHexString(SHA256.HashData(body))}");
Console.WriteLine($"output {chunked.Length,8} bytes  sha256 {Convert.ToHexString(SHA256.HashData(chunked))}");
return 0;
