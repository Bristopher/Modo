// SPDX-License-Identifier: AGPL-3.0-or-later

import {randomUUID} from 'node:crypto';
import type {UserID} from '../BrandedTypes';
import {BatchBuilder} from '../database/CassandraQueryExecution';
import {RiskAssessments, RiskAssessmentsByUser} from '../Tables';
import type {RiskAssessment} from './RiskTypes';

export interface IRiskAssessmentRepository {
	recordAssessment(params: {
		userId: UserID | null;
		ip: string;
		email: string | null;
		locale: string | null;
		assessment: RiskAssessment;
	}): Promise<void>;
}

export class CassandraRiskAssessmentRepository implements IRiskAssessmentRepository {
	async recordAssessment(params: {
		userId: UserID | null;
		ip: string;
		email: string | null;
		locale: string | null;
		assessment: RiskAssessment;
	}): Promise<void> {
		const assessmentId = randomUUID();
		const createdAt = new Date();
		const batch = new BatchBuilder();
		batch.addPrepared(
			RiskAssessments.insert({
				assessment_id: assessmentId,
				created_at: createdAt,
				user_id: params.userId,
				ip: params.ip,
				email: params.email,
				locale: params.locale,
				risk_level: params.assessment.level,
				risk_score: params.assessment.riskScore,
				suspicious: params.assessment.suspicious,
				method: params.assessment.method,
				model_used: params.assessment.modelUsed,
				recommended_action: params.assessment.recommendedAction,
				reasoning: params.assessment.reasoning,
				signals_json: JSON.stringify(params.assessment.signals),
			}),
		);
		if (params.userId != null) {
			batch.addPrepared(
				RiskAssessmentsByUser.insert({
					user_id: params.userId,
					created_at: createdAt,
					assessment_id: assessmentId,
					risk_level: params.assessment.level,
					risk_score: params.assessment.riskScore,
				}),
			);
		}
		await batch.execute(false);
	}
}
