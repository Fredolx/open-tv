/**
 * Beats TV - Premium IPTV Player
 * Copyright (C) 2026 Beats TV Team
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program; if not, write to the Free Software Foundation, Inc.,
 * 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
 *
 * This project is a fork of Open TV by Fredolx.
 */

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
