{{/*
SPDX-License-Identifier: AGPL-3.0-or-later
*/}}

{{- define "svc-common.service" -}}
apiVersion: v1
kind: Service
metadata:
  name: {{ .Values.svc.name }}
  namespace: {{ .Values.global.namespace }}
spec:
  selector:
    app: {{ .Values.svc.name }}
  ports:
    - port: {{ .Values.svc.router.port }}
      name: http
{{- end -}}
