class zulip::simonai {
  include zulip::supervisor

  file { "${zulip::common::supervisor_conf_dir}/simon-ai.conf":
    ensure  => file,
    require => [
      Package[supervisor],
    ],
    owner   => 'root',
    group   => 'root',
    mode    => '0644',
    content => template('zulip/supervisor/simon-ai.conf.erb'),
    notify  => Service[supervisor],
  }
}