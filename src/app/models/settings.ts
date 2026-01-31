export class Settings {
  recording_path?: string;
  use_stream_caching?: boolean;
  mpv_params?: string;
  default_view?: number;
  volume?: number;
  refresh_on_start?: boolean;
  restream_port?: number;
  enable_tray_icon?: boolean;
  zoom?: number;
  default_sort?: number;
  enable_hwdec?: boolean;
  always_ask_save?: boolean;
  enable_gpu?: boolean;
  use_single_column?: boolean;
  max_text_lines?: number;
  compact_mode?: boolean;
  refresh_interval?: number;
  last_refresh?: number;
  enhanced_video?: boolean;
  theme?: number; // 0=Clay-Mation, 1=Smooth Glass, 2=Matrix Terminal
}
