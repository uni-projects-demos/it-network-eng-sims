export type StatRow = Readonly<{
  lbl: string;
  val: string | number;
  title?: string;
  dir?: "ltr" | "rtl" | "auto";
}>;
