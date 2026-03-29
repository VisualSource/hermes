import { TRANSFORMERS } from "@lexical/markdown";
import { NOTIFY } from "./user-at/lexical";
import { EMOJI } from "./emoji/lexical";

export const FULL_TRANSFORMS = [...TRANSFORMERS, NOTIFY, EMOJI];
