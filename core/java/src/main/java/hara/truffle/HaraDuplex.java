package hara.truffle;

import hara.lang.protocol.IClose;
import hara.lang.protocol.IPromise;
import hara.lang.protocol.IStream;

/** Native bidirectional transport: one pull stream plus Promise-returning sends. */
final class HaraDuplex implements IClose {
  private final HaraContext context;
  private final IStream receive;
  private final Object send;
  private final Object close;
  private boolean closed;

  HaraDuplex(HaraContext context, IStream receive, Object send, Object close) {
    this.context = context;
    this.receive = receive;
    this.send = send;
    this.close = close;
  }

  synchronized IStream receive() {
    return receive;
  }

  synchronized Object send(Object value) {
    if (closed) return context.rejectedPromise("duplex/closed: cannot send on a closed Duplex");
    try {
      Object result = context.invokeCallable(send, new Object[] {value});
      return HaraBox.unwrap(result) instanceof IPromise ? result : context.completedPromise(result);
    } catch (Throwable error) {
      return context.rejectedPromise(error.getMessage());
    }
  }

  @Override
  public synchronized void close() throws Exception {
    if (closed) return;
    closed = true;
    receive.close();
    if (close != null) context.invokeCallable(close, new Object[0]);
  }

  @Override
  public synchronized String toString() {
    return "#<duplex " + (closed ? "closed" : "open") + ">";
  }
}
