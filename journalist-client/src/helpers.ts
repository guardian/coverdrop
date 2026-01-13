// Takes a Date object and returns a formatted date/time string
// If the message is from today, only show the time, otherwise show the date and time
export const formatDateTime = (date: Date) => {
  const today = new Date();
  if (
    date.getDate() === today.getDate() &&
    date.getMonth() === today.getMonth() &&
    date.getFullYear() === today.getFullYear()
  ) {
    return date.toLocaleTimeString();
  } else {
    return date.toLocaleDateString() + " " + date.toLocaleTimeString();
  }
};

// Takes a datetime string and returns a formatted date/time string
export const formatDateTimeString = (dateTime: string) => {
  const date = new Date(dateTime);
  return formatDateTime(date);
};
