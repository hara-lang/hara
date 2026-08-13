package hara.truffle;

import com.oracle.truffle.api.interop.ArityException;
import com.oracle.truffle.api.interop.InteropLibrary;
import com.oracle.truffle.api.interop.TruffleObject;
import com.oracle.truffle.api.library.ExportLibrary;
import com.oracle.truffle.api.library.ExportMessage;

@ExportLibrary(InteropLibrary.class)
public class HaraType implements TruffleObject {
  private final String name;
  private final String[] fields;

  public HaraType(String name, String[] fields) {
    this.name = name;
    this.fields = fields.clone();
  }

  public int arity() {
    return fields.length;
  }

  public Object construct(Object[] values) throws ArityException {
    requireArity(values.length);
    return new HaraStruct(this, values);
  }

  final void requireArity(int actual) throws ArityException {
    if (actual != fields.length) {
      throw ArityException.create(fields.length, fields.length, actual);
    }
  }

  int fieldIndex(String field) {
    for (int i = 0; i < fields.length; i++) {
      if (fields[i].equals(field)) {
        return i;
      }
    }
    return -1;
  }

  public String name() {
    return name;
  }

  String[] fields() {
    return fields.clone();
  }

  @ExportMessage
  boolean isExecutable() {
    return true;
  }

  @ExportMessage
  Object execute(Object[] arguments) throws ArityException {
    return HaraBox.export(construct(arguments));
  }

  @ExportMessage
  @com.oracle.truffle.api.CompilerDirectives.TruffleBoundary
  Object toDisplayString(boolean allowSideEffects) {
    return "#<type " + name + ">";
  }

  @Override
  public String toString() {
    return "#<type " + name + ">";
  }
}
