{{/*
SPDX-License-Identifier: AGPL-3.0-or-later
*/}}

{{- define "svc-common.headless-service" -}}
apiVersion: v1
kind: Service
metadata:
  name: {{ .Values.svc.name }}-shard-headless
  namespace: {{ .Values.global.namespace }}
spec:
  clusterIP: None
  publishNotReadyAddresses: true
  selector:
    app: {{ .Values.svc.name }}-shard
  ports:
    - port: {{ .Values.svc.shard.port }}
      name: http
{{- end -}}
