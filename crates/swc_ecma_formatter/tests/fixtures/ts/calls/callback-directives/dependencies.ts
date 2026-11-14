function C() {
  useEffect(() => {
    // @ts-ignore
    missing();
  }, [alpha, beta, gamma, delta, epsilon, zeta, eta, theta]);
}
