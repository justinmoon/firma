package it.casatta

import android.app.Activity
import android.content.Intent
import android.os.Bundle
import android.util.Log
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import android.widget.Toast
import androidx.appcompat.app.AlertDialog
import androidx.appcompat.app.AppCompatActivity
import androidx.recyclerview.widget.LinearLayoutManager
import androidx.recyclerview.widget.RecyclerView
import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.module.kotlin.KotlinModule
import it.casatta.ListActivity.Companion.network
import kotlinx.android.synthetic.main.activity_psbt.*
import java.io.File
import java.io.Serializable
import java.util.*
import kotlin.collections.ArrayList


class PSBTActivity : AppCompatActivity() {
    val mapper = ObjectMapper().registerModule(KotlinModule())
    val inputsAdapter = TxInOutAdapter()
    val outputsAdapter = TxInOutAdapter()
    val itemsAdapter = DescItemAdapter()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_psbt)

        inputs.layoutManager = LinearLayoutManager(this)
        inputs.adapter = inputsAdapter

        outputs.layoutManager = LinearLayoutManager(this)
        outputs.adapter = outputsAdapter

        val network = intent.getStringExtra(C.NETWORK)
        val psbtString = intent.getStringExtra(C.PSBT)
        Log.d("PSBT", "$network $psbtString")
        val psbtJson = mapper.readValue(psbtString, Rust.PsbtJsonOutput::class.java)
        val psbtFileDir = "$filesDir/$network/psbts/${psbtJson.psbt.name}/"
        val psbtFileName = "$psbtFileDir/psbt.json"
        val psbtPretty = Rust().print(filesDir.toString(),network, psbtFileName)

        val psbtTitle = "$network PSBT: ${psbtJson.psbt.name}"
        title = psbtTitle
        view_qr.setOnClickListener { QrActivity.comeHere(this, psbtTitle, psbtJson.qr_files ) }
        select.setOnClickListener {
            val returnIntent = Intent()
            returnIntent.putExtra(C.RESULT, psbtJson.psbt.name)
            setResult(Activity.RESULT_OK, returnIntent)
            finish()
        }
        delete.setOnClickListener {
            val dialog: AlertDialog = AlertDialog.Builder(this)
                .setTitle("Are you sure?")
                .setMessage("Delete ${psbtJson.psbt.name}")
                .setPositiveButton("Ok") { dialog, which ->
                    File(psbtFileDir).deleteRecursively()
                    val intent = Intent(this, MainActivity::class.java)
                    intent.flags = Intent.FLAG_ACTIVITY_CLEAR_TASK or Intent.FLAG_ACTIVITY_NEW_TASK
                    startActivity(intent)
                }
                .setNegativeButton("Cancel", null)
                .create()
            dialog.show()
        }

        for (i in psbtPretty.inputs.indices) {
            val input = psbtPretty.inputs[i]
            inputsAdapter.list.add(TxInOutItem("input #$i",input.outpoint!!, input.value, "${input.path} ${input.wallet}"))
        }

        for (i in psbtPretty.outputs.indices) {
            val output = psbtPretty.outputs[i]
            outputsAdapter.list.add(TxInOutItem("output #$i",output.address!!, output.value, "${output.path} ${output.wallet}"))
        }

        items.layoutManager = LinearLayoutManager(this)
        items.adapter = itemsAdapter

        val info = psbtPretty.info.joinToString()
        if (!info.isEmpty()) {
            itemsAdapter.list.add(DescItem("Info", info))
        }
        val formattedRate = String.format(Locale.US, "%.2f sat/vB", psbtPretty.fee.rate) ;
        itemsAdapter.list.add(DescItem("Fee", psbtPretty.fee.absolute_fmt))
        itemsAdapter.list.add(DescItem("Fee rate", formattedRate))
        itemsAdapter.list.add(DescItem("Balances", psbtPretty.balances))
        itemsAdapter.list.add(DescItem("Estimated size", "${psbtPretty.size.estimated} bytes"))
        itemsAdapter.list.add(DescItem("Unsigned size", "${psbtPretty.size.unsigned} bytes"))
        itemsAdapter.list.add(DescItem("PSBT size", "${psbtPretty.size.psbt} bytes"))
        itemsAdapter.list.add(DescItem("PSBT", psbtJson.psbt.psbt))

    }
}

data class TxInOutItem(val index: String, val title: String, val value: String, val description: String?): Serializable

class TxInOutAdapter() : RecyclerView.Adapter<TxInOutItemHolder>(){

    val list: ArrayList<TxInOutItem> = ArrayList()

    override fun getItemCount():Int{
        return list.size
    }

    override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): TxInOutItemHolder {
        var item = LayoutInflater.from(parent?.context).inflate(R.layout.txinout_item, parent, false)
        return TxInOutItemHolder(item)
    }
    override fun onBindViewHolder(holder: TxInOutItemHolder, position: Int) {
        var item = list[position]
        holder?.update(item)
    }
}

class TxInOutItemHolder(itemView: View): RecyclerView.ViewHolder(itemView) {
    private val index = itemView.findViewById<TextView>(R.id.index)
    private val title = itemView.findViewById<TextView>(R.id.title)
    private val value = itemView.findViewById<TextView>(R.id.value)
    private val description = itemView.findViewById<TextView>(R.id.description)

    fun update(item: TxInOutItem) {
        index.text = item.index
        title.text = item.title
        value.text = item.value
        description.text = item.description
    }
}




