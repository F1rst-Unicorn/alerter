/* alerter: Alerter to chat servers
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

package de.njsm.alerter.test;

public class Config {

    public static final String CLIENT_BINARY_PATH = "de.njsm.alerter.client.binary";

    public static final String BINARY_PATH = "de.njsm.alerter.server.binary";

    public static final String CONFIG_PATH = "de.njsm.alerter.config";

    public static final String CLIENT_CONFIG_PATH = "de.njsm.alerter.client.config";

    public static final String LOG_CONFIG_PATH = "de.njsm.alerter.logconfig";

    public static final String SOCKET_PATH = "alert.sock";

    public static String getClientBinaryPath() {
        return System.getProperty(CLIENT_BINARY_PATH);
    }

    public static String getServerBinaryPath() {
        return System.getProperty(BINARY_PATH);
    }

    public static String getConfigPath() {
        return System.getProperty(CONFIG_PATH);
    }

    public static String getClientConfigPath() {
        return System.getProperty(CLIENT_CONFIG_PATH);
    }

    public static String getLogConfigPath() {
        return System.getProperty(LOG_CONFIG_PATH);
    }




}
