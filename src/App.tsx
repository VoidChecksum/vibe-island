import { useEffect } from "react";
import { useStore } from "./store/useStore";
import { NotchPanel } from "./components/notch/NotchPanel";

export default function App() {
  const init = useStore((s) => s.init);

  useEffect(() => {
    init();
  }, [init]);

  return (
    <div className="w-screen h-screen flex justify-center">
      <NotchPanel />
    </div>
  );
}
