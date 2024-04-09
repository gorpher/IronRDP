// <auto-generated/> by Diplomat

#pragma warning disable 0105
using System;
using System.Runtime.InteropServices;

using Devolutions.IronRdp.Diplomat;
#pragma warning restore 0105

namespace Devolutions.IronRdp;

#nullable enable

public partial class PduHint: IDisposable
{
    private unsafe Raw.PduHint* _inner;

    /// <summary>
    /// Creates a managed <c>PduHint</c> from a raw handle.
    /// </summary>
    /// <remarks>
    /// Safety: you should not build two managed objects using the same raw handle (may causes use-after-free and double-free).
    /// <br/>
    /// This constructor assumes the raw struct is allocated on Rust side.
    /// If implemented, the custom Drop implementation on Rust side WILL run on destruction.
    /// </remarks>
    public unsafe PduHint(Raw.PduHint* handle)
    {
        _inner = handle;
    }

    /// <exception cref="IronRdpException"></exception>
    /// <returns>
    /// A <c>OptionalUsize</c> allocated on Rust side.
    /// </returns>
    public OptionalUsize FindSize(byte[] bytes)
    {
        unsafe
        {
            if (_inner == null)
            {
                throw new ObjectDisposedException("PduHint");
            }
            nuint bytesLength = (nuint)bytes.Length;
            fixed (byte* bytesPtr = bytes)
            {
                Raw.ConnectorFfiResultBoxOptionalUsizeBoxIronRdpError result = Raw.PduHint.FindSize(_inner, bytesPtr, bytesLength);
                if (!result.isOk)
                {
                    throw new IronRdpException(new IronRdpError(result.Err));
                }
                Raw.OptionalUsize* retVal = result.Ok;
                return new OptionalUsize(retVal);
            }
        }
    }

    /// <summary>
    /// Returns the underlying raw handle.
    /// </summary>
    public unsafe Raw.PduHint* AsFFI()
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

            Raw.PduHint.Destroy(_inner);
            _inner = null;

            GC.SuppressFinalize(this);
        }
    }

    ~PduHint()
    {
        Dispose();
    }
}