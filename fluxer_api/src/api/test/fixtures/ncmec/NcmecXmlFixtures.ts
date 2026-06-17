// SPDX-License-Identifier: AGPL-3.0-or-later

export const INVALID_REPORT_XML = `<?xml version="1.0" encoding="UTF-8"?>
<report xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
        xsi:noNamespaceSchemaLocation="https://report.cybertip.org/ispws/xsd">
    <reporter>
        <reportingPerson>
            <firstName>John</firstName>
            <lastName>Smith</lastName>
            <email>jsmith@example.com</email>
        </reportingPerson>
    </reporter>
</report>`;
export const MALFORMED_REPORT_XML = `<?xml version="1.0" encoding="UTF-8"?>
<report xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
        xsi:noNamespaceSchemaLocation="https://report.cybertip.org/ispws/xsd">
    <incidentSummary>
        <incidentType>Child Pornography (possession, manufacture, and distribution)</incidentType>
        <incidentDateTime>2012-10-15T08:00:00-07:00</incidentDateTime>
    </incidentSummary>
    <reporter>
        <reportingPerson>
            <firstName>John</firstName>
            <lastName>Smith</lastName>
            <email>jsmith@example.com</email>
        </reportingPerson>
    </reporter>
</report`;
