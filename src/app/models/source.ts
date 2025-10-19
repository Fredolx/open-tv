import { SourceType } from "./sourceType";

export class Source {
  id?: number;
  name?: string;
  url?: string;
  username?: string;
  password?: string;
  source_type?: SourceType;
  enabled?: boolean;
  use_tvg_id?: boolean;
  user_agent?: string;
  max_streams?: number;
}
