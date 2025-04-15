export class EPG {
  epg_id!: string;
  title!: string;
  description!: string;
  start_time!: string;
  start_timestamp!: number;
  end_time!: string;
  has_archive!: boolean;
  now_playing!: boolean;
  timeshift_url?: string;
}
