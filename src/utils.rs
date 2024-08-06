use std::{fs, io, path::PathBuf};

use figment::providers::{Format, Toml};

use crate::config::{Config, ASSET_DIR, IN_DIR, TEMPLATE_DIR};

pub fn get_config() -> Config {
    Config::figment()
        .merge(Toml::file("sitdown.yaml"))
        .extract()
        .unwrap()
}

pub fn create_new(name: String) -> io::Result<()> {
    let in_dir = PathBuf::from(&name).join(IN_DIR);
    let asset_dir = PathBuf::from(&name).join(ASSET_DIR);
    let template_dir = PathBuf::from(&name).join(TEMPLATE_DIR);
    fs::create_dir(&name)?;
    fs::create_dir(&in_dir)?;
    fs::create_dir(in_dir.join("content"))?;
    fs::create_dir(&asset_dir)?;
    fs::create_dir(asset_dir.join("css"))?;
    fs::create_dir(&template_dir)?;

    fs::write(in_dir.join("home.md"), "This is the homepage")?;
    fs::write(
        in_dir.join("content").join("Something.md"),
        "This is some content",
    )?;
    fs::write(
        template_dir.join("layout.jinja"),
        r#"
<!doctype html>
<html>

<head>
	<title>{% block title %}Title{% endblock %}</title>
	<link rel="stylesheet" type="text/css" href="/assets/css/default.css" />
</head>

<body>
	<div class="header">
		<a href="/index.html">
			<h1>Title</h1>
		</a>
	</div>
	<nav>
		<div class="navbar">
			{% for header in headers %}
			<a href={{ header.link }}>{{ header.title }}</a>
			{% endfor %}
		</div>

	</nav>
	<div class="content">
		{% block body %}{% endblock %}
	</div>
	<div class="footer">
		Footer stuff like attribution.
	</div>
</body>

</html>
    "#,
    )?;
    fs::write(
        template_dir.join("content.jinja"),
        r#"
{% extends "layout" %}
{% block title %}{{ super() }} | {{title }} {% endblock %}
{% block body %}
<h1>{{ title }}</h1>
<p>{{ content }}</p>
{% endblock %}
    "#,
    )?;
    fs::write(
        template_dir.join("entries.jinja"),
        r#"
{% extends "layout" %}
{% block title %}{{ super() }} | {{ title }} {% endblock %}
{% block body %}
<h1>{{ title }}</h1>
<div class="collapse">
{% for item in entries recursive %}
	{% if item.children %}
		<details>
		<summary>
			<a href={{ item.link }}>{{ item.title }}</a>
		</summary>
			<div class="children">{{ loop(item.children) }}</div>
		</details>
	{% else %}
		<div class="detail"><a href={{ item.link }}>{{ item.title }}</a></div>
	{% endif %}
{% endfor %}
</div>

{% endblock %}
    "#,
    )?;
    fs::write(
        asset_dir.join("css").join("default.css"),
        r#"
* {
	box-sizing: border-box;
}

.navbar {
	background-color: #333;
	overflow: hidden;
}

.navbar a {
	float: left;
	color: #f2f2f2;
	text-align: center;
	padding: 14px 16px;
	text-decoration: none;
	font-size: 17px;
}

.navbar a:hover {
	background-color: #ddd;
	color: black;
}

.content {
	background-color: white;
	padding: 20px;
	margin-top: 20px;
}

.header {
	padding: 30px;
	text-align: center;
	background: white;
}

.header a {
	color: black;
	text-decoration: inherit;	
}

.footer {
	margin-top: 20px;
	padding: 20px;
	background: #ddd;
}

.collapse summary {
	background: #eee;
}

.collapse .children {
	margin-left: 20px;
}

body {
	padding: 10px;
	background: #f1f1f1;
}
    "#,
    )?;

    Ok(())
}
