/* alerter: Alerter to Slack
 * Copyright (C) 2019  The alerter developers
 *
 * This file is part of the alerter program suite.
 *
 * alerter is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * alerter is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

package de.njsm.alerter.test.client;

import org.apache.commons.io.IOUtils;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

import java.io.IOException;
import java.nio.charset.Charset;
import java.util.ArrayList;
import java.util.Map;
import java.util.Optional;
import java.util.TreeMap;

public class Alert {

    private static final Logger LOG = LogManager.getLogger(Alert.class);

    public static final String BINARY_PATH = "de.njsm.alerter.binary";

    public static final String CONFIG_PATH = "de.njsm.alerter.config";

    public static final String LOG_CONFIG_PATH = "de.njsm.alerter.logconfig";

    private String text;

    private String title;

    private Optional<String> channel;

    private Optional<String> titleLink;

    private Optional<String> level;

    private final Map<String, String> fields;

    public Alert() {
        this.text = "";
        this.title = "";
        this.channel = Optional.empty();
        this.titleLink = Optional.empty();
        this.level = Optional.empty();
        this.fields = new TreeMap<>();
    }

    public static AlertBuilder build() {
        return new AlertBuilder();
    }

    private void call() {
        ArrayList<String> command = new ArrayList<>();

        command.add(getBinaryPath());
        command.add("-C");
        command.add(getConfigPath());
        command.add("-v");
        command.add(getLogConfigPath());

        command.add(title);
        command.add(text);

        channel.ifPresent(v -> {
            command.add("-c");
            command.add(v);
        });

        titleLink.ifPresent(v -> {
            command.add("-t");
            command.add(v);
        });

        level.ifPresent(v -> {
            command.add("-l");
            command.add(v);
        });

        fields.forEach((k,v) -> {
            command.add("-f");
            command.add(k + ":" + v);
        });

        try {
            Process p = Runtime.getRuntime().exec(command.toArray(new String[0]));
            p.waitFor();
            String stdout = IOUtils.toString(p.getInputStream(), Charset.defaultCharset());
            String stderr = IOUtils.toString(p.getErrorStream(), Charset.defaultCharset());
            LOG.debug("command: {}", command);
            if (!stdout.isEmpty())
                LOG.debug("stdout: {}", stdout);
            if (!stderr.isEmpty())
                LOG.debug("stderr: {}", stderr);
        } catch (InterruptedException | IOException e) {
            LOG.error("Failed to execute alert command", e);
        }
    }

    private String getBinaryPath() {
        return System.getProperty(BINARY_PATH);
    }

    private String getConfigPath() {
        return System.getProperty(CONFIG_PATH);
    }

    private String getLogConfigPath() {
        return System.getProperty(LOG_CONFIG_PATH);
    }

    public static class AlertBuilder {

        private final Alert product;

        public AlertBuilder() {
            product = new Alert();
        }

        public AlertBuilder withText(String text) {
            product.text = text;
            return this;
        }

        public AlertBuilder withTitle(String title) {
            product.title = title;
            return this;
        }

        public AlertBuilder withChannel(String channel) {
            product.channel = Optional.of(channel);
            return this;
        }

        public AlertBuilder withTitleLink(String titleLink) {
            product.titleLink = Optional.of(titleLink);
            return this;
        }

        public AlertBuilder withLevel(String level) {
            product.level = Optional.of(level);
            return this;
        }

        public AlertBuilder withField(String key, String value) {
            product.fields.put(key, value);
            return this;
        }

        public void call() {
            product.call();
        }
    }
}
