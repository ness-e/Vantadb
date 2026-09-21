// DESKTOP-29: the shared poll hook must run ONE vanta_metrics call per 4s tick
// regardless of consumer count, and stop the interval when the last unmounts.
import { act, render } from "@testing-library/react";
import { vi, describe, it, expect, beforeEach, afterEach } from "vitest";
import { useMetricsPoll } from "./useMetricsPoll";
import { metrics } from "../vanta";

vi.mock("../vanta", () => ({
  metrics: vi.fn(),
  vantaErrorMessage: (e: unknown) => String(e),
}));

const mocked = vi.mocked(metrics);
let n = 0;
mocked.mockImplementation(async () => ({ process_rss_bytes: ++n }) as never);

function Probe({ onSnap }: { onSnap?: (h: ReturnType<typeof useMetricsPoll>) => void }) {
  const s = useMetricsPoll();
  onSnap?.(s);
  return null;
}

beforeEach(() => {
  vi.useFakeTimers();
  n = 0;
  mocked.mockClear();
});

afterEach(() => {
  vi.useRealTimers();
});

describe("useMetricsPoll", () => {
  it("makes one call per tick for multiple consumers", async () => {
    const snaps: unknown[] = [];
    const view = render(
      <>
        <Probe onSnap={(s) => snaps.push(s.history.length)} />
        <Probe />
        <Probe />
      </>,
    );

    await act(async () => {}); // initial tick resolves
    expect(mocked).toHaveBeenCalledTimes(1);

    await act(async () => {
      await vi.advanceTimersByTimeAsync(4000);
    });
    expect(mocked).toHaveBeenCalledTimes(2); // not 2-3 per extra consumer

    view.unmount();
  });

  it("stops the interval after the last consumer unmounts", async () => {
    const a = render(<Probe />);
    const b = render(<Probe />);
    await act(async () => {});
    expect(mocked).toHaveBeenCalledTimes(1);

    a.unmount(); // one consumer left — poller must stay alive
    await act(async () => {
      await vi.advanceTimersByTimeAsync(4000);
    });
    expect(mocked).toHaveBeenCalledTimes(2);

    b.unmount(); // last consumer — poller stops
    await act(async () => {
      await vi.advanceTimersByTimeAsync(12000);
    });
    expect(mocked).toHaveBeenCalledTimes(2);
  });

  it("caps history at 12 entries", async () => {
    let last = 0;
    const view = render(<Probe onSnap={(s) => (last = s.history.length)} />);
    await act(async () => {});
    for (let i = 0; i < 14; i++) {
      await act(async () => {
        await vi.advanceTimersByTimeAsync(4000);
      });
    }
    expect(last).toBe(12);
    view.unmount();
  });
});
