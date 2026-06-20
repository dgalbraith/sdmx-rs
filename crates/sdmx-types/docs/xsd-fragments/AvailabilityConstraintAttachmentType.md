<details>
<summary>XSD contract: <code>AvailabilityConstraintAttachmentType</code> (SDMX 3.1)</summary>

```xml
	<xs:complexType name="AvailabilityConstraintAttachmentType">
		<xs:annotation>
				<xs:documentation>AvailabilityConstraintAttachmentType describes a collection of references to data-related artefacts, for which availability is provided.</xs:documentation>
		</xs:annotation>
		<xs:complexContent>
			<xs:restriction base="ConstraintAttachmentType">
				<xs:choice>
					<xs:element name="DataStructure" type="common:DataStructureReferenceType" maxOccurs="1"/>
					<xs:element name="Dataflow" type="common:DataflowReferenceType" maxOccurs="1"/>
					<xs:element name="ProvisionAgreement" type="common:ProvisionAgreementReferenceType" maxOccurs="1"/>
				</xs:choice>
			</xs:restriction>
		</xs:complexContent>
	</xs:complexType>
```

</details>
