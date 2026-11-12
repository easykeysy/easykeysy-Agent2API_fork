import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'

/**
 * and ：clsx and ，tailwind-merge after 。
 *
 * 1 use className —— use need can default，
 * tailwind-merge by 「」 not is CSS （ in 
 * @layer utilities ， by ， not can ）。
 */
export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs))
}
