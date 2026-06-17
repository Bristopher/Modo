// SPDX-License-Identifier: AGPL-3.0-or-later

import {deleteOneOrMany, fetchOne, upsertOne} from '../../../database/CassandraQueryExecution';
import type {PasswordChangeTicketRow} from '../../../database/types/AuthTypes';
import {PasswordChangeTickets} from '../../../Tables';

const FETCH_TICKET_CQL = PasswordChangeTickets.selectCql({
	where: PasswordChangeTickets.where.eq('ticket'),
	limit: 1,
});

export class PasswordChangeRepository {
	async createTicket(row: PasswordChangeTicketRow): Promise<void> {
		await upsertOne(PasswordChangeTickets.insert(row));
	}

	async updateTicket(row: PasswordChangeTicketRow): Promise<void> {
		await upsertOne(PasswordChangeTickets.upsertAll(row));
	}

	async findTicket(ticket: string): Promise<PasswordChangeTicketRow | null> {
		return await fetchOne<PasswordChangeTicketRow>(FETCH_TICKET_CQL, {ticket});
	}

	async deleteTicket(ticket: string): Promise<void> {
		await deleteOneOrMany(PasswordChangeTickets.deleteByPk({ticket}));
	}
}
