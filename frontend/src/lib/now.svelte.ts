// A clock that ticks every second, so relative times and countdowns stay
// live. Create it while a component initializes.
export class Now {
  current = $state(Date.now());

  constructor(intervalMs = 1000) {
    $effect(() => {
      const timer = setInterval(() => {
        this.current = Date.now();
      }, intervalMs);
      return () => clearInterval(timer);
    });
  }
}
