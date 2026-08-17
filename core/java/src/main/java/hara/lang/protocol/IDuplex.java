package hara.lang.protocol;

/** Bidirectional stream with graceful and exceptional termination. */
public interface IDuplex extends IStream, IStreamWrite, IAbort {}
