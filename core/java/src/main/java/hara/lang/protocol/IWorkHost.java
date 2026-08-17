package hara.lang.protocol;

/** Process-owned host that admits work and resolves live run handles. */
public interface IWorkHost extends IComponent {
  IWorkRun workSubmit(Object work, Object input, Object options);

  IWorkRun workResolve(Object reference);
}
