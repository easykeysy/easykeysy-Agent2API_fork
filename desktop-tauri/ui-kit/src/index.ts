/**
 * @agent2api/ui —— has 。
 *
 * （ not npm ）， is desktop-tauri/ui-islands；
 * styles/globals.css，must @source directory。
 *
 * **、props 、 and as ** 1 for shadcn/ui 
 * （Base UI that 1 ）： by shadcn can in ， not 。
 * **** then 1 （styles/theme.css）， and has 1 。
 *
 * new when by this ： shadcn has no has ， has API ，
 * not need prop（Input up has `icon`， already by InputGroup ）。
 */

export { Button, buttonVariants, type ButtonProps } from './button'
export { Input } from './input'
export { Textarea } from './textarea'
export {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput,
  InputGroupText,
  InputGroupTextarea,
} from './input-group'
export { Badge, BadgeDot, badgeVariants, type BadgeProps } from './badge'
export { Avatar, AvatarImage, AvatarFallback } from './avatar'
export { Spinner } from './spinner'
export { Skeleton } from './skeleton'
export { Label } from './label'
export { Switch, type SwitchProps } from './switch'
export { Checkbox } from './checkbox'
export { Progress, ProgressTrack, ProgressIndicator, ProgressLabel, ProgressValue } from './progress'
export { Tooltip, TooltipProvider, TooltipTrigger, TooltipContent, TooltipArrow, type TooltipContentProps } from './tooltip'
export {
  Dialog,
  DialogTrigger,
  DialogPortal,
  DialogClose,
  DialogOverlay,
  DialogContent,
  DialogTitle,
  DialogDescription,
  DialogHeader,
  DialogBody,
  DialogFooter,
  DialogSection,
} from './dialog'
export {
  AlertDialog,
  AlertDialogTrigger,
  AlertDialogPortal,
  AlertDialogOverlay,
  AlertDialogContent,
  AlertDialogHeader,
  AlertDialogBody,
  AlertDialogFooter,
  AlertDialogBanner,
  AlertDialogTitle,
  AlertDialogDescription,
} from './alert-dialog'
export {
  Table,
  TableHeader,
  TableBody,
  TableFooter,
  TableHead,
  TableRow,
  TableCell,
  TableCaption,
  type TableHeadProps,
  type TableSortDirection,
} from './table'
export {
  RadioGroup,
  RadioGroupItem,
  type RadioGroupProps,
  type RadioGroupItemProps,
} from './radio-group'
export {
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
  SelectGroup,
  SelectLabel,
  SelectSeparator,
} from './select'
export {
  SegmentedControl,
  type SegmentedControlOption,
  type SegmentedControlProps,
} from './segmented-control'
export { Toggle, toggleVariants, type ToggleProps } from './toggle'
export {
  MultiSelect,
  type MultiSelectOption,
  type MultiSelectProps,
} from './multi-select'
export {
  Popover,
  PopoverTrigger,

  PopoverClose,
  PopoverContent,
  PopoverHeader,
  PopoverTitle,
  PopoverDescription,
} from './popover'
export { Pager, type PagerProps } from './pager'
export { NavList, NavItem, navItemVariants, type NavItemProps } from './nav-item'
export {
  Sheet,
  SheetTrigger,
  SheetClose,
  SheetPortal,
  SheetOverlay,
  SheetContent,
  SheetTitle,
  SheetDescription,
  SheetHeader,
  SheetBody,
  SheetFooter,
} from './sheet'
export { ResponsiveSheet, useMediaQuery, type ResponsiveSheetProps } from './responsive-sheet'
export { HtmlContent } from './html-content'
export { Meter, type MeterProps } from './meter'
export { MetricCard, MetricLabel, MetricValue, MetricGrid } from './metric'
export { confirmDialog } from './confirm'
export { StatusDot, type StatusDotProps } from './status-dot'
export { Verification } from './verification'
export { cn } from './lib/cn'
export { Card, CardHeader, CardTitle, CardDescription, CardAction, CardContent, CardFooter, type CardProps } from './card'
export { Item, ItemGroup, ItemContent, ItemTitle, ItemDescription, ItemMedia, ItemActions } from './item'
export { Field, FieldGroup, FieldSet, FieldLegend, FieldLabel, FieldContent, FieldDescription, FieldError, type FieldProps } from './field'
export { Alert, AlertTitle, AlertDescription, Empty, EmptyTitle, EmptyDescription } from './feedback'
export { Collapsible, CollapsibleTrigger, CollapsibleContent } from './collapsible'
export { Toast, ToastProvider, ToastTitle, ToastDescription, ToastClose, Toaster, toastManager, createToastManager } from './toast'
export { AppFrame, Sidebar, SidebarHeader, SidebarContent, SidebarFooter, SidebarGroupLabel, PageHeader, PageTitle, PageDescription, BottomNavigation, BottomNavigationItem, WindowTitleBar, WindowControl } from './layout'
