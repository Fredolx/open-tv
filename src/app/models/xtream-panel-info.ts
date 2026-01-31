export interface XtreamUserInfo {
  username?: string;
  status?: string;
  active_cons?: string;
  is_trial?: string;
  created_at?: string;
  exp_date?: string;
  max_connections?: string;
}

export interface XtreamServerInfo {
  url?: string;
  port?: string;
  https_port?: string;
  server_protocol?: string;
  rtmp_port?: string;
  timezone?: string;
  timestamp_now?: number;
  time_now?: string;
}

export interface XtreamPanelInfo {
  user_info: XtreamUserInfo;
  server_info: XtreamServerInfo;
}
