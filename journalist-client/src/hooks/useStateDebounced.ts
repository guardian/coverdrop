import React, { useEffect, useState } from "react";

export const useStateDebounced = <T>(
  initialValue: T,
  delay: number,
): [T, T, React.Dispatch<React.SetStateAction<T>>] => {
  const [value, setValue] = useState<T>(initialValue);
  const [debouncedValue, setDebouncedValue] = useState<T>(initialValue);

  useEffect(() => {
    const timeout = setTimeout(() => setDebouncedValue(value), delay);
    return () => clearTimeout(timeout);
  }, [value]);

  return [value, debouncedValue, setValue];
};
