package hara.lang.protocol;

/** Bidirectional stream with graceful and exceptional termination. */
public interface IStreamDuplex extends IStream, IStreamWrite, IAbort {}
