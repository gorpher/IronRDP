// <auto-generated/> by Diplomat

#pragma warning disable 0105
using System;
using System.Runtime.InteropServices;

using Devolutions.IronRdp.Diplomat;
#pragma warning restore 0105

namespace Devolutions.IronRdp;

#nullable enable

public partial class CliprdrBackendFactory: IDisposable
{
    private unsafe Raw.CliprdrBackendFactory* _inner;

    /// <summary>
    /// Creates a managed <c>CliprdrBackendFactory</c> from a raw handle.
    /// </summary>
    /// <remarks>
    /// Safety: you should not build two managed objects using the same raw handle (may causes use-after-free and double-free).
    /// <br/>
    /// This constructor assumes the raw struct is allocated on Rust side.
    /// If implemented, the custom Drop implementation on Rust side WILL run on destruction.
    /// </remarks>
    public unsafe CliprdrBackendFactory(Raw.CliprdrBackendFactory* handle)
    {
        _inner = handle;
    }

    /// <returns>
    /// A <c>Cliprdr</c> allocated on Rust side.
    /// </returns>
    public Cliprdr BuildCliprdr()
    {
        unsafe
        {
            if (_inner == null)
            {
                throw new ObjectDisposedException("CliprdrBackendFactory");
            }
            Raw.Cliprdr* retVal = Raw.CliprdrBackendFactory.BuildCliprdr(_inner);
            return new Cliprdr(retVal);
        }
    }

    /// <summary>
    /// Returns the underlying raw handle.
    /// </summary>
    public unsafe Raw.CliprdrBackendFactory* AsFFI()
    {
        return _inner;
    }

    /// <summary>
    /// Destroys the underlying object immediately.
    /// </summary>
    public void Dispose()
    {
        unsafe
        {
            if (_inner == null)
            {
                return;
            }

            Raw.CliprdrBackendFactory.Destroy(_inner);
            _inner = null;

            GC.SuppressFinalize(this);
        }
    }

    ~CliprdrBackendFactory()
    {
        Dispose();
    }
}