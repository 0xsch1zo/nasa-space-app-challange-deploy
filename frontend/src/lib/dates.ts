import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
import 'dayjs/locale/pl';

dayjs.extend(relativeTime);
dayjs.locale('pl');

export const fmtDate = (iso: string) => dayjs(iso).format('D MMM YYYY');
export const fmtDateTime = (iso: string) => dayjs(iso).format('D MMM YYYY, HH:mm');
export const fmtShort = (iso: string) => dayjs(iso).format('DD.MM');
export const fromNow = (iso: string) => dayjs(iso).fromNow();
export { dayjs };
